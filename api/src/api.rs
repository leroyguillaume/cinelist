//! HTTP surface. Everything lives under `/api` so that the web front and the API
//! can share one origin behind a plain path-prefix route.

mod films;
mod invitations;
mod lists;
mod participants;
mod selections;
mod sessions;
mod users;

use std::sync::Arc;

use aide::{
    axum::ApiRouter,
    openapi::{ApiKeyLocation, OpenApi, SecurityScheme, Tag},
    transform::TransformOpenApi,
};
use axum::{
    Extension, Json, Router,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::warn;
use validator::{Validate, ValidationError};

use crate::{
    auth::{BEARER_SCHEME, COOKIE_SCHEME, TOKEN_COOKIE},
    error::AppResult,
    state::AppState,
};

const DEFAULT_PER_PAGE: u32 = 50;
const MAX_PER_PAGE: u32 = 100;

const TAGS: [(&str, &str); 7] = [
    ("Sessions", "Signing in and out."),
    (
        "Invitations",
        "Accounts are created by the administrator and activated through a single-use link.",
    ),
    (
        "Users",
        "Account management, reserved to the administrator.",
    ),
    ("Lists", "Film lists and their settings."),
    (
        "Participants",
        "People taking turns in a list's selections.",
    ),
    ("Films", "Films of a list, and their track record."),
    (
        "Selections",
        "Elimination rounds that pick the next film to watch.",
    ),
];

pub fn router(state: AppState) -> Router {
    aide::generate::extract_schemas(true);
    let mut openapi = OpenApi::default();
    ApiRouter::new()
        .merge(sessions::routes())
        .merge(invitations::routes())
        .merge(users::routes())
        .merge(lists::routes())
        .merge(participants::routes())
        .merge(films::routes())
        .merge(selections::routes())
        .finish_api_with(&mut openapi, describe)
        .route("/api/openapi.json", get(serve_openapi))
        .route("/api/docs", get(serve_docs))
        .route("/healthz", get(|| async { StatusCode::NO_CONTENT }))
        .route("/readyz", get(readyz))
        .layer(Extension(Arc::new(openapi)))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

fn describe(api: TransformOpenApi<'_>) -> TransformOpenApi<'_> {
    let api = api
        .title("CineList API")
        .description(
            "Keeps film lists and runs the elimination rounds that pick what to watch next.",
        )
        .security_scheme(
            BEARER_SCHEME,
            SecurityScheme::Http {
                scheme: "bearer".to_owned(),
                bearer_format: Some("JWT".to_owned()),
                description: Some("Token returned by `POST /api/v1/sessions`.".to_owned()),
                extensions: IndexMap::default(),
            },
        )
        .security_scheme(
            COOKIE_SCHEME,
            SecurityScheme::ApiKey {
                location: ApiKeyLocation::Cookie,
                name: TOKEN_COOKIE.to_owned(),
                description: Some(
                    "The same token, set as a cookie by `POST /api/v1/sessions`.".to_owned(),
                ),
                extensions: IndexMap::default(),
            },
        );
    TAGS.iter().fold(api, |api, (name, description)| {
        api.tag(Tag {
            name: (*name).to_owned(),
            description: Some((*description).to_owned()),
            ..Default::default()
        })
    })
}

async fn serve_openapi(Extension(openapi): Extension<Arc<OpenApi>>) -> impl IntoResponse {
    Json(openapi)
}

async fn serve_docs() -> Html<&'static str> {
    Html(include_str!("../static/docs.html"))
}

async fn readyz(State(state): State<AppState>) -> StatusCode {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => StatusCode::NO_CONTENT,
        Err(err) => {
            warn!(error = %err, "database unreachable");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

#[derive(Deserialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PageQuery {
    /// 1-based page number.
    #[validate(range(min = 1))]
    page: Option<u32>,
    /// Items per page, 50 by default, 100 at most.
    #[validate(range(min = 1, max = 100))]
    per_page: Option<u32>,
}

pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
}

impl Pagination {
    pub fn limit(&self) -> i64 {
        i64::from(self.per_page)
    }

    pub fn offset(&self) -> i64 {
        i64::from(self.page - 1) * i64::from(self.per_page)
    }

    pub fn wrap<T>(self, items: Vec<T>, total: i64) -> PageResponse<T> {
        PageResponse {
            items,
            page: self.page,
            per_page: self.per_page,
            total,
        }
    }
}

impl PageQuery {
    pub fn pagination(&self) -> AppResult<Pagination> {
        self.validate()?;
        Ok(Pagination {
            page: self.page.unwrap_or(1),
            per_page: self.per_page.unwrap_or(DEFAULT_PER_PAGE).min(MAX_PER_PAGE),
        })
    }
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
// Without it, schemars 1 names the instances `PageResponse`, `PageResponse2`, …
// in whatever order the routes register them.
#[schemars(rename = "PageResponse_for_{T}")]
pub struct PageResponse<T> {
    items: Vec<T>,
    page: u32,
    per_page: u32,
    /// Number of items across every page.
    total: i64,
}

pub fn not_blank(value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::new("blank"))
    } else {
        Ok(())
    }
}
