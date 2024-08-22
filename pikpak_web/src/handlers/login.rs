use std::fmt::{Debug, Display};

use axum::Json;
use base64::prelude::*;
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use pikpak_core::api::login::ApiLoginReq;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use utoipa::{ToResponse, ToSchema};

use crate::{
    handlers::{get_pikpak_client, JWT_SECRET},
    utils::token::{Claims, TokenData},
};

use super::{BaseResp, CIPHER};

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct LoginReq {
    email: String,
    password: String,
}

impl Display for LoginReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginReq")
            .field("email", &self.email)
            .field("password", &"********")
            .finish()
    }
}

impl Debug for LoginReq {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoginReq")
            .field("email", &self.email)
            .field("password", &"********")
            .finish()
    }
}

#[derive(Serialize, Deserialize, ToSchema, ToResponse)]
pub struct LoginResp {
    #[serde(flatten)]
    base_resp: BaseResp,
    token: String,
}

#[utoipa::path(
    post,
    path = "",
    request_body = LoginReq,
    responses(
        (status = 200, description = "Login success, return jwt", body = LoginResp),
        (status = 400, description = "Login failed, return error message", body = BaseResp)
    )
)]
pub async fn login(Json(req): Json<LoginReq>) -> Result<Json<LoginResp>, BaseResp> {
    debug!("[login] request: {:?}", req);

    let expiration = Utc::now() + Duration::days(1);
    get_pikpak_client()
        .login(&ApiLoginReq {
            username: req.email.clone(),
            password: req.password.clone().into(),
        })
        .await
        .map_err(|e| {
            error!("[login] login error: {}", e);
            BaseResp::with_error(e)
        })?;
    let token_data = TokenData {
        email: req.email,
        password: req.password.into(),
    };
    let token_data = serde_json::to_vec(&token_data).map_err(|e| {
        error!("[login] serialize token error: {}", e);
        BaseResp::with_error(e)
    })?;
    let encrypted_token_data = CIPHER.encrypt(&token_data).map_err(|e| {
        error!("[login] encrypt token error: {}", e);
        BaseResp::with_error(e)
    })?;
    let token = BASE64_STANDARD.encode(encrypted_token_data);
    let jwt = encode(
        &Header::default(),
        &Claims {
            sub: token,
            exp: expiration.timestamp() as usize,
        },
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )
    .map_err(|e| {
        error!("[login] encode jwt error: {}", e);
        BaseResp::with_error(e)
    })?;

    Ok(Json(LoginResp {
        base_resp: Default::default(),
        token: jwt,
    }))
}

#[cfg(feature = "utoipa")]
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(login),
    components(schemas(LoginReq, LoginResp, BaseResp), responses(LoginResp, BaseResp))
)]
pub(super) struct LoginApi;
