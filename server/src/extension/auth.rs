use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};
use base64::prelude::*;

use crate::{
    handlers::{BaseResp, CIPHER},
    utils::token::Claims,
};

pub struct AuthExtractor(pub Claims);

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
        let token = BASE64_STANDARD.decode(token).map_err(|e| {
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
                BaseResp::with_error(format!("Failed to decrypt JWT: {}", e)),
            )
        })?;

        let claim: Claims = serde_json::from_slice(&claims).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                BaseResp::with_error(format!("Failed to parse JWT claims: {}", e)),
            )
        })?;
        Ok(AuthExtractor(claim))
    }
}
