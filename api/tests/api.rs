use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{Method, Request, StatusCode, header},
};
use chrono::Duration;
use cinelist_api::{AppState, JwtKeys, MIGRATOR, Settings, bootstrap_admin, router};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

const ADMIN_PASSWORD: &str = "admin-password";
const JWT_SECRET: &[u8] = b"integration-tests-jwt-secret-0123456789";

struct Client {
    app: Router,
    cookie: Option<String>,
    bearer: Option<String>,
}

struct Reply {
    status: StatusCode,
    body: Value,
}

impl Client {
    async fn new(pool: PgPool) -> Self {
        bootstrap_admin(&pool, "admin", ADMIN_PASSWORD.to_owned())
            .await
            .expect("bootstrap admin");
        let state = AppState {
            pool,
            settings: Arc::new(Settings {
                token_ttl: Duration::days(30),
                invitation_ttl: Duration::days(7),
                cookie_secure: false,
                jwt: JwtKeys::from_secret(JWT_SECRET),
            }),
        };
        Self {
            app: router(state),
            cookie: None,
            bearer: None,
        }
    }

    fn anonymous(&self) -> Self {
        Self {
            app: self.app.clone(),
            cookie: None,
            bearer: None,
        }
    }

    async fn send(&mut self, method: Method, uri: &str, body: Option<Value>) -> Reply {
        let mut request = Request::builder().method(method).uri(uri);
        if let Some(cookie) = &self.cookie {
            request = request.header(header::COOKIE, cookie);
        }
        if let Some(token) = &self.bearer {
            request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let request = match body {
            Some(body) => request
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string())),
            None => request.body(Body::empty()),
        }
        .expect("request");
        let response = self.app.clone().oneshot(request).await.expect("response");
        if let Some(set_cookie) = response.headers().get(header::SET_COOKIE) {
            let pair = set_cookie
                .to_str()
                .expect("cookie")
                .split(';')
                .next()
                .expect("pair");
            self.cookie = Some(pair.to_owned());
        }
        let status = response.status();
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into()))
        };
        Reply { status, body }
    }

    async fn get(&mut self, uri: &str) -> Reply {
        self.send(Method::GET, uri, None).await
    }

    async fn post(&mut self, uri: &str, body: Value) -> Reply {
        self.send(Method::POST, uri, Some(body)).await
    }

    async fn put(&mut self, uri: &str, body: Value) -> Reply {
        self.send(Method::PUT, uri, Some(body)).await
    }

    async fn delete(&mut self, uri: &str) -> Reply {
        self.send(Method::DELETE, uri, None).await
    }

    async fn sign_in(&mut self, username: &str, password: &str) -> Reply {
        self.post(
            "/api/v1/sessions",
            json!({
                "username": username,
                "password": password,
            }),
        )
        .await
    }

    /// Signs in as a new regular user, created and invited by the administrator.
    async fn user(&self, username: &str) -> Self {
        let mut admin = self.anonymous();
        assert_eq!(
            admin.sign_in("admin", ADMIN_PASSWORD).await.status,
            StatusCode::CREATED
        );
        let created = admin
            .post("/api/v1/users", json!({ "username": username }))
            .await;
        assert_eq!(created.status, StatusCode::CREATED, "{:?}", created.body);
        let token = created.body["invitation"]["token"]
            .as_str()
            .expect("token")
            .to_owned();
        let mut user = self.anonymous();
        let accepted = user
            .put(
                &format!("/api/v1/invitations/{token}/acceptance"),
                json!({ "password": "correct horse" }),
            )
            .await;
        assert_eq!(accepted.status, StatusCode::OK, "{:?}", accepted.body);
        user
    }

    async fn create_list(&mut self, participants: &[&str], films: &[&str]) -> Value {
        let reply = self
            .post(
                "/api/v1/lists",
                json!({
                    "name": "Soirées ciné",
                    "participants": participants,
                    "films": films,
                }),
            )
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{:?}", reply.body);
        reply.body
    }
}

fn id(value: &Value) -> String {
    value["id"].as_str().expect("id").to_owned()
}

fn film_id(list: &Value, title: &str) -> String {
    list["films"]
        .as_array()
        .expect("films")
        .iter()
        .find(|film| film["title"] == title)
        .map(id)
        .expect("film")
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn serves_the_openapi_document_and_its_reference(pool: PgPool) {
    let mut client = Client::new(pool).await;
    let openapi = client.get("/api/openapi.json").await;
    assert_eq!(openapi.status, StatusCode::OK);
    assert!(openapi.body["paths"]["/api/v1/lists"].is_object());
    let docs = client.get("/api/docs").await;
    assert_eq!(docs.status, StatusCode::OK);
    assert!(docs.body.as_str().expect("html").contains("api-reference"));
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn names_each_page_schema_after_its_items(pool: PgPool) {
    let mut client = Client::new(pool).await;
    let openapi = client.get("/api/openapi.json").await;
    let schema = &openapi.body["paths"]["/api/v1/lists"]["get"]["responses"]["200"]["content"]["application/json"]
        ["schema"]["$ref"];
    assert_eq!(
        schema,
        "#/components/schemas/PageResponse_for_ListSummaryResponse"
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn rejects_anonymous_requests(pool: PgPool) {
    let mut client = Client::new(pool).await;
    let reply = client.get("/api/v1/lists").await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(reply.body["code"], "unauthenticated");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn rejects_a_wrong_password(pool: PgPool) {
    let mut client = Client::new(pool).await;
    let reply = client.sign_in("admin", "not the password").await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(reply.body["code"], "invalid_credentials");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn signs_in_and_out(pool: PgPool) {
    let mut client = Client::new(pool).await;
    assert_eq!(
        client.sign_in("ADMIN", ADMIN_PASSWORD).await.status,
        StatusCode::CREATED
    );
    let current = client.get("/api/v1/sessions/current").await;
    assert_eq!(current.body["user"]["isAdmin"], true);
    assert_eq!(
        client.delete("/api/v1/sessions/current").await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        client.get("/api/v1/sessions/current").await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn invitations_activate_accounts_once(pool: PgPool) {
    let client = Client::new(pool).await;
    let mut admin = client.anonymous();
    admin.sign_in("admin", ADMIN_PASSWORD).await;
    let created = admin
        .post("/api/v1/users", json!({ "username": "camille" }))
        .await;
    let token = created.body["invitation"]["token"]
        .as_str()
        .expect("token")
        .to_owned();

    let mut visitor = client.anonymous();
    let invitation = visitor.get(&format!("/api/v1/invitations/{token}")).await;
    assert_eq!(invitation.body["username"], "camille");
    let body = json!({ "password": "correct horse" });
    let uri = format!("/api/v1/invitations/{token}/acceptance");
    assert_eq!(visitor.put(&uri, body.clone()).await.status, StatusCode::OK);
    assert_eq!(visitor.put(&uri, body).await.status, StatusCode::NOT_FOUND);

    let mut camille = client.anonymous();
    assert_eq!(
        camille.sign_in("camille", "correct horse").await.status,
        StatusCode::CREATED
    );

    let users = admin.get("/api/v1/users").await;
    assert_eq!(users.body["total"], 2);
    assert_eq!(users.body["items"][1]["status"], "active");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn a_new_invitation_resets_the_password(pool: PgPool) {
    let client = Client::new(pool).await;
    let _camille = client.user("camille").await;
    let mut admin = client.anonymous();
    admin.sign_in("admin", ADMIN_PASSWORD).await;
    let users = admin.get("/api/v1/users").await;
    let camille_id = id(&users.body["items"][1]);
    let reset = admin
        .post(
            &format!("/api/v1/users/{camille_id}/invitations"),
            json!({}),
        )
        .await;
    assert_eq!(reset.status, StatusCode::CREATED);
    let token = reset.body["token"].as_str().expect("token");
    let mut visitor = client.anonymous();
    visitor
        .put(
            &format!("/api/v1/invitations/{token}/acceptance"),
            json!({ "password": "battery staple" }),
        )
        .await;
    let mut camille = client.anonymous();
    assert_eq!(
        camille.sign_in("camille", "correct horse").await.status,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        camille.sign_in("camille", "battery staple").await.status,
        StatusCode::CREATED
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn only_the_administrator_manages_users(pool: PgPool) {
    let client = Client::new(pool).await;
    let mut camille = client.user("camille").await;
    assert_eq!(
        camille.get("/api/v1/users").await.status,
        StatusCode::FORBIDDEN
    );

    let mut admin = client.anonymous();
    admin.sign_in("admin", ADMIN_PASSWORD).await;
    let users = admin.get("/api/v1/users").await;
    let admin_id = id(&users.body["items"][0]);
    let reply = admin.delete(&format!("/api/v1/users/{admin_id}")).await;
    assert_eq!(reply.body["code"], "admin_managed");
    let taken = admin
        .post("/api/v1/users", json!({ "username": "Camille" }))
        .await;
    assert_eq!(taken.body["code"], "username_taken");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn a_selection_needs_as_many_turns_for_everyone(pool: PgPool) {
    let client = Client::new(pool).await;
    let mut alex = client.user("alex").await;
    let list = alex.create_list(&["Alex", "Sam"], &["Heat", "Drive"]).await;
    let list_id = id(&list);
    assert_eq!(list["filmsToAdd"], 1);
    let uneven = alex
        .post(&format!("/api/v1/lists/{list_id}/selections"), json!({}))
        .await;
    assert_eq!(uneven.status, StatusCode::CONFLICT);
    assert_eq!(uneven.body["code"], "uneven_film_count");

    alex.post(
        &format!("/api/v1/lists/{list_id}/films"),
        json!({ "title": "Alien" }),
    )
    .await;
    let list = alex.get(&format!("/api/v1/lists/{list_id}")).await.body;
    assert_eq!(list["filmsToAdd"], 0);

    alex.post(
        &format!("/api/v1/lists/{list_id}/participants"),
        json!({ "name": "Noé" }),
    )
    .await;
    let list = alex.get(&format!("/api/v1/lists/{list_id}")).await.body;
    assert_eq!(list["filmCount"], 3);
    assert_eq!(list["filmsToAdd"], 1);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn lists_are_private(pool: PgPool) {
    let client = Client::new(pool).await;
    let mut alex = client.user("alex").await;
    let list = alex.create_list(&["Alex", "Sam"], &["Stalker"]).await;
    let mut camille = client.user("camille").await;
    let reply = camille.get(&format!("/api/v1/lists/{}", id(&list))).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(camille.get("/api/v1/lists").await.body["total"], 0);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn a_list_is_never_empty(pool: PgPool) {
    let client = Client::new(pool).await;
    let mut alex = client.user("alex").await;
    let list = alex.create_list(&["Alex", "Sam"], &["Stalker"]).await;
    let reply = alex
        .delete(&format!("/api/v1/films/{}", film_id(&list, "Stalker")))
        .await;
    assert_eq!(reply.body["code"], "last_film");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn a_selection_runs_to_a_watched_film(pool: PgPool) {
    let client = Client::new(pool).await;
    let mut alex = client.user("alex").await;
    let list = alex.create_list(&["Alex", "Sam"], &["Heat", "Drive"]).await;
    let list_id = id(&list);

    let incomplete = alex
        .post(&format!("/api/v1/lists/{list_id}/selections"), json!({}))
        .await;
    assert_eq!(incomplete.body["code"], "uneven_film_count");
    alex.post(
        &format!("/api/v1/lists/{list_id}/films"),
        json!({ "title": "Alien" }),
    )
    .await;

    let list = alex.get(&format!("/api/v1/lists/{list_id}")).await.body;
    let started = alex
        .post(&format!("/api/v1/lists/{list_id}/selections"), json!({}))
        .await;
    assert_eq!(started.status, StatusCode::CREATED);
    let selection = started.body;
    let selection_id = id(&selection);
    assert_eq!(selection["phase"], "eliminating");
    assert_eq!(selection["currentParticipant"]["name"], "Alex");

    let during = alex
        .post(
            &format!("/api/v1/lists/{list_id}/films"),
            json!({ "title": "Her" }),
        )
        .await;
    assert_eq!(during.body["code"], "selection_in_progress");

    let eliminations = format!("/api/v1/selections/{selection_id}/eliminations");
    let first = alex
        .post(&eliminations, json!({ "filmId": film_id(&list, "Heat") }))
        .await;
    assert_eq!(first.body["eliminations"][0]["place"], 3);
    assert_eq!(first.body["eliminations"][0]["participantName"], "Alex");
    assert_eq!(first.body["currentParticipant"]["name"], "Sam");

    let undone = alex
        .delete(&format!(
            "{eliminations}/{}",
            id(&first.body["eliminations"][0])
        ))
        .await;
    assert_eq!(undone.status, StatusCode::NO_CONTENT);
    let again = alex
        .post(&eliminations, json!({ "filmId": film_id(&list, "Heat") }))
        .await;
    assert_eq!(again.body["currentParticipant"]["name"], "Sam");
    let twice = alex
        .post(&eliminations, json!({ "filmId": film_id(&list, "Heat") }))
        .await;
    assert_eq!(twice.body["code"], "film_already_eliminated");

    let last = alex
        .post(&eliminations, json!({ "filmId": film_id(&list, "Drive") }))
        .await;
    assert_eq!(last.body["phase"], "awaitingVerdict");
    assert_eq!(last.body["candidate"]["title"], "Alien");
    assert_eq!(last.body["candidate"]["place"], 1);

    let verdicts = format!("/api/v1/selections/{selection_id}/verdicts");
    let abandoned = alex
        .post(
            &verdicts,
            json!({
                "filmId": film_id(&list, "Alien"),
                "outcome": "abandoned",
            }),
        )
        .await;
    assert_eq!(abandoned.body["candidate"]["title"], "Drive");
    assert_eq!(abandoned.body["candidate"]["place"], 2);
    let stale = alex
        .post(
            &verdicts,
            json!({
                "filmId": film_id(&list, "Alien"),
                "outcome": "watched",
            }),
        )
        .await;
    assert_eq!(stale.body["code"], "not_awaiting_verdict");
    let watched = alex
        .post(
            &verdicts,
            json!({
                "filmId": film_id(&list, "Drive"),
                "outcome": "watched",
            }),
        )
        .await;
    assert_eq!(watched.body["phase"], "completed");
    assert_eq!(watched.body["verdicts"][0]["outcome"], "abandoned");
    assert_eq!(watched.body["verdicts"][1]["filmTitle"], "Drive");

    let list = alex.get(&format!("/api/v1/lists/{list_id}")).await.body;
    assert_eq!(list["filmCount"], 1);
    assert!(list["activeSelection"].is_null());
    let sam_id = list["participants"][1]["id"].clone();
    assert_eq!(list["nextStarterId"], sam_id);

    let history = alex
        .get(&format!("/api/v1/lists/{list_id}/selections"))
        .await;
    assert_eq!(history.body["total"], 1);
    assert_eq!(
        history.body["items"][0]["eliminations"][1]["filmTitle"],
        "Heat"
    );

    let stats = alex
        .get(&format!(
            "/api/v1/films/{}/statistics",
            film_id(&list, "Heat")
        ))
        .await;
    assert_eq!(stats.body["selectionCount"], 1);
    assert_eq!(stats.body["worstPlace"], 3);
    assert_eq!(stats.body["eliminatedBy"][0]["participantName"], "Alex");
    assert_eq!(stats.body["recentRuns"][0]["winnerTitle"], "Drive");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn a_selection_can_be_cancelled_before_any_verdict(pool: PgPool) {
    let client = Client::new(pool).await;
    let mut alex = client.user("alex").await;
    let list = alex
        .create_list(&["Alex", "Sam"], &["Heat", "Drive", "Alien"])
        .await;
    let list_id = id(&list);
    let selection = alex
        .post(&format!("/api/v1/lists/{list_id}/selections"), json!({}))
        .await
        .body;
    let uri = format!("/api/v1/selections/{}", id(&selection));
    assert_eq!(alex.delete(&uri).await.status, StatusCode::NO_CONTENT);
    assert_eq!(alex.get(&uri).await.status, StatusCode::NOT_FOUND);
    let list = alex.get(&format!("/api/v1/lists/{list_id}")).await.body;
    assert_eq!(list["nextStarterId"], list["participants"][0]["id"]);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn a_bearer_token_authenticates_without_the_cookie(pool: PgPool) {
    let client = Client::new(pool).await;
    let mut browser = client.anonymous();
    let signed_in = browser.sign_in("admin", ADMIN_PASSWORD).await;
    let token = signed_in.body["token"].as_str().expect("token").to_owned();
    assert!(signed_in.body["expiresAt"].is_string());

    let mut script = client.anonymous();
    script.bearer = Some(token);
    let current = script.get("/api/v1/sessions/current").await;
    assert_eq!(current.status, StatusCode::OK);
    assert_eq!(current.body["user"]["username"], "admin");
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn a_token_signed_with_another_key_is_rejected(pool: PgPool) {
    let client = Client::new(pool).await;
    let mut admin = client.anonymous();
    let token = admin.sign_in("admin", ADMIN_PASSWORD).await.body["token"]
        .as_str()
        .expect("token")
        .to_owned();
    let (head, rest) = token.split_once('.').expect("header");
    let (payload, _) = rest.split_once('.').expect("payload");
    let mut forger = client.anonymous();
    forger.bearer = Some(format!("{head}.{payload}.c2lnbmF0dXJl"));
    let reply = forger.get("/api/v1/sessions/current").await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn accepting_an_invitation_revokes_earlier_tokens(pool: PgPool) {
    let client = Client::new(pool).await;
    let mut camille = client.user("camille").await;
    assert_eq!(camille.get("/api/v1/lists").await.status, StatusCode::OK);

    let mut admin = client.anonymous();
    admin.sign_in("admin", ADMIN_PASSWORD).await;
    let users = admin.get("/api/v1/users").await;
    let camille_id = id(&users.body["items"][1]);
    let reset = admin
        .post(
            &format!("/api/v1/users/{camille_id}/invitations"),
            json!({}),
        )
        .await;
    let token = reset.body["token"].as_str().expect("token");
    client
        .anonymous()
        .put(
            &format!("/api/v1/invitations/{token}/acceptance"),
            json!({ "password": "battery staple" }),
        )
        .await;

    assert_eq!(
        camille.get("/api/v1/lists").await.status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrator = "MIGRATOR")]
async fn a_deleted_user_token_is_rejected(pool: PgPool) {
    let client = Client::new(pool).await;
    let mut camille = client.user("camille").await;
    let mut admin = client.anonymous();
    admin.sign_in("admin", ADMIN_PASSWORD).await;
    let users = admin.get("/api/v1/users").await;
    let camille_id = id(&users.body["items"][1]);
    admin.delete(&format!("/api/v1/users/{camille_id}")).await;

    assert_eq!(
        camille.get("/api/v1/lists").await.status,
        StatusCode::UNAUTHORIZED
    );
}
