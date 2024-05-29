use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};
use base64::prelude::*;
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{
    handlers::{BaseResp, CIPHER, JWT_SECRET},
    utils::token::{Claims, TokenData},
};

pub struct AuthExtractor(pub TokenData);

#[async_trait]
impl<S> FromRequestParts<S> for AuthExtractor
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, BaseResp);

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let auth = parts.headers.get(AUTHORIZATION).ok_or((
            StatusCode::BAD_REQUEST,
            BaseResp::with_error("`AUTHORIZATION` header is missing"),
        ))?;

        let auth = auth.to_str().map_err(|e| {
            log::error!("Failed to parse `AUTHORIZATION` header: {}", e);
            (
                StatusCode::BAD_REQUEST,
                BaseResp::with_error(format!(
                    "`AUTHORIZATION` header is not a valid string, err: {}",
                    e
                )),
            )
        })?;

        if !auth.starts_with("Bearer ") {
            return Err((
                StatusCode::BAD_REQUEST,
                BaseResp::with_error("`AUTHORIZATION` header is not a Bearer token"),
            ));
        }

        let token = auth.trim_start_matches("Bearer ");
        let token = decode::<Claims>(
            token,
            &DecodingKey::from_secret(JWT_SECRET.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                BaseResp::with_error(format!("Failed to decode token: {}", e)),
            )
        })?;

        if token.claims.exp < chrono::Utc::now().timestamp() as usize {
            return Err((
                StatusCode::BAD_REQUEST,
                BaseResp::with_error("token has expired"),
            ));
        }

        let token = BASE64_STANDARD
            .decode(token.claims.sub.as_bytes())
            .map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    BaseResp::with_error(format!(
                        "`AUTHORIZATION` header is not valid base64, err: {}",
                        e
                    )),
                )
            })?;

        let claims = CIPHER.decrypt(&token).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                BaseResp::with_error(format!("Failed to decrypt token: {}", e)),
            )
        })?;

        let claim: TokenData = serde_json::from_slice(&claims).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                BaseResp::with_error(format!("Failed to parse token: {}", e)),
            )
        })?;

        Ok(AuthExtractor(claim))
    }
}

pub(crate) struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme(
            "jwt",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::HttpBuilder::new()
                    .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                    .build(),
            ),
        )
    }
}
