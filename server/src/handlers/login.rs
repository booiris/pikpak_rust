use axum::Json;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::utils::jwt::Claims;

use super::{BaseResp, CIPHER, JWT_SECRET};

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct LoginReq {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginResp {
    #[serde(flatten)]
    base_resp: BaseResp,
    jwt: String,
}

pub async fn login(Json(req): Json<LoginReq>) -> Result<Json<LoginResp>, BaseResp> {
    let expiration = Utc::now() + Duration::hours(1);
    let claims = Claims {
        exp: expiration.timestamp() as usize,
        email: req.email,
        password: req.password,
    };

    let claims = serde_json::to_vec(&claims).map_err(|e| {
        log::error!("[login] serialize claims error: {}", e);
        BaseResp::with_error(e)
    })?;

    let encrypted_claims = CIPHER.encrypt(&claims).map_err(|e| {
        log::error!("[login] encrypt claims error: {}", e);
        BaseResp::with_error(e)
    })?;

    let token = encode(
        &Header::default(),
        &encrypted_claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )
    .map_err(|e| {
        log::error!("[login] encode token error: {}", e);
        BaseResp::with_error(e)
    })?;

    Ok(Json(LoginResp {
        base_resp: Default::default(),
        jwt: token,
    }))
}
