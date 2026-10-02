use std::sync::Arc;

use chrono::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub settings: Arc<Settings>,
}

pub struct Settings {
    /// Lifetime of an issued JWT; it is not extended by use.
    pub token_ttl: Duration,
    pub invitation_ttl: Duration,
    pub cookie_secure: bool,
    pub jwt: JwtKeys,
}

/// HMAC keys signing and verifying the JWTs (HS256).
pub struct JwtKeys {
    pub(crate) encoding: EncodingKey,
    pub(crate) decoding: DecodingKey,
}

impl JwtKeys {
    #[must_use]
    pub fn from_secret(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}
