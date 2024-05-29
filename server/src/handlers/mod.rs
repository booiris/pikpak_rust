use std::sync::OnceLock;

use axum::response::IntoResponse;
use lazy_static::lazy_static;
use pikpak_core::PkiPakApiClient;

use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::{extension::auth::SecurityAddon, utils::token::Cipher};

pub mod login;
pub mod remote_list;

#[derive(Serialize, Deserialize, ToSchema, ToResponse, Clone)]
pub struct BaseResp {
    code: i32,
    message: String,
}

impl IntoResponse for BaseResp {
    fn into_response(self) -> axum::response::Response {
        let code = self.code;
        let mut resp = axum::Json(self).into_response();
        if code != 0 {
            *resp.status_mut() = axum::http::StatusCode::BAD_REQUEST;
        }
        resp
    }
}

impl BaseResp {
    pub fn with_error(e: impl ToString) -> Self {
        Self {
            code: -1,
            message: e.to_string(),
        }
    }
}

impl Default for BaseResp {
    fn default() -> Self {
        Self {
            code: 0,
            message: "success".to_string(),
        }
    }
}

lazy_static! {
    pub(crate) static ref CIPHER: Cipher = Cipher::new();
    pub(crate) static ref JWT_SECRET: String = {
        let rng = rand::thread_rng();
        rng.sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    };
}

pub(crate) static PIKPAK_CORE_CLIENT: OnceLock<PkiPakApiClient> = OnceLock::new();

pub(crate) fn get_pikpak_client() -> &'static PkiPakApiClient {
    PIKPAK_CORE_CLIENT
        .get()
        .expect("pikpak core client not initialized")
}

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/api/login", api = login::LoginApi),
        (path = "/api/remote_list", api = remote_list::RemoteListApi),
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;
