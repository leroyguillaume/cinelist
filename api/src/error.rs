use aide::OperationOutput;
use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use schemars::JsonSchema;
use serde::Serialize;
use tracing::{debug, error};

/// Stable, machine-readable reason of a failure; the web front translates it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    NotFound,
    Unauthenticated,
    Forbidden,
    InvalidCredentials,
    ValidationFailed,
    UsernameTaken,
    AdminManaged,
    TooManyParticipants,
    LastParticipant,
    ListFull,
    UnevenFilmCount,
    LastFilm,
    SelectionInProgress,
    NotEliminating,
    FilmAlreadyEliminated,
    NotLatestElimination,
    NotAwaitingVerdict,
    VerdictStarted,
    Internal,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{code:?}")]
    Api { status: StatusCode, code: ErrorCode },
    #[error("validation failed")]
    Validation(#[from] validator::ValidationErrors),
    #[error("database error")]
    Database(#[from] sqlx::Error),
    #[error("password hashing error")]
    PasswordHash(argon2::password_hash::Error),
    #[error("token signing error")]
    Jwt(jsonwebtoken::errors::Error),
}

impl AppError {
    pub fn not_found() -> Self {
        Self::api(StatusCode::NOT_FOUND, ErrorCode::NotFound)
    }

    pub fn unauthenticated() -> Self {
        Self::api(StatusCode::UNAUTHORIZED, ErrorCode::Unauthenticated)
    }

    pub fn forbidden() -> Self {
        Self::api(StatusCode::FORBIDDEN, ErrorCode::Forbidden)
    }

    pub fn conflict(code: ErrorCode) -> Self {
        Self::api(StatusCode::CONFLICT, code)
    }

    pub fn unprocessable(code: ErrorCode) -> Self {
        Self::api(StatusCode::UNPROCESSABLE_ENTITY, code)
    }

    fn api(status: StatusCode, code: ErrorCode) -> Self {
        Self::Api { status, code }
    }
}

/// RFC 9457 problem details.
#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProblemResponse {
    #[serde(rename = "type")]
    kind: String,
    title: String,
    status: u16,
    code: ErrorCode,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    invalid_fields: Vec<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, invalid_fields) = match self {
            Self::Api { status, code } => {
                debug!(%status, ?code, "request rejected");
                (status, code, Vec::new())
            }
            Self::Validation(errors) => {
                let mut fields: Vec<String> =
                    errors.errors().keys().map(ToString::to_string).collect();
                fields.sort();
                debug!(?fields, "request failed validation");
                (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    ErrorCode::ValidationFailed,
                    fields,
                )
            }
            Self::Database(err) => internal("database", &err),
            Self::PasswordHash(err) => internal("password hashing", &err),
            Self::Jwt(err) => internal("token signing", &err),
        };
        let code_name = serde_json::to_value(code)
            .ok()
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .unwrap_or_default();
        let body = ProblemResponse {
            kind: format!("urn:cinelist:error:{code_name}"),
            title: status.canonical_reason().unwrap_or("Error").to_owned(),
            status: status.as_u16(),
            code,
            invalid_fields,
        };
        let mut response = (status, Json(body)).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        response
    }
}

fn internal(
    source: &'static str,
    err: &impl std::fmt::Display,
) -> (StatusCode, ErrorCode, Vec<String>) {
    error!(source, error = %err, "internal error");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorCode::Internal,
        Vec::new(),
    )
}

impl OperationOutput for AppError {
    type Inner = ();
}

pub type AppResult<T> = Result<T, AppError>;
