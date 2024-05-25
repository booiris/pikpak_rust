use axum::Json;
use base64::prelude::*;
use chrono::{Duration, Utc};
use pikpak_core::api::login::ApiLoginReq;
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::{handlers::get_pikpak_client, utils::token::Claims};

use super::{BaseResp, CIPHER};

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct LoginReq {
    email: String,
    password: String,
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
    log::trace!("[login] request: {:?}", req);

    let expiration = Utc::now() + Duration::hours(1);
    let token = get_pikpak_client()
        .login(ApiLoginReq {
            username: req.email.clone(),
            password: req.password.clone(),
        })
        .await
        .map_err(|e| {
            log::error!("[login] login error: {}", e);
            BaseResp::with_error(e)
        })?;
    let claims = Claims {
        exp: expiration.timestamp() as usize,
        email: req.email,
        password: req.password,
        oauth2_token: token.access_token,
        oauth2_refresh_token: token.refresh_token,
    };

    let claims = serde_json::to_vec(&claims).map_err(|e| {
        log::error!("[login] serialize claims error: {}", e);
        BaseResp::with_error(e)
    })?;

    let encrypted_claims = CIPHER.encrypt(&claims).map_err(|e| {
        log::error!("[login] encrypt claims error: {}", e);
        BaseResp::with_error(e)
    })?;

    Ok(Json(LoginResp {
        base_resp: Default::default(),
        token: BASE64_STANDARD.encode(encrypted_claims),
    }))
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(login),
    components(schemas(LoginReq, LoginResp, BaseResp), responses(LoginResp, BaseResp))
)]
pub(super) struct LoginApi;
