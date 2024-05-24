use axum::response::IntoResponse;
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, Rng};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::utils::jwt::Cipher;

pub mod login;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub(crate) struct BaseResp {
    #[schema(example = "0")]
    code: i32,
    #[schema(example = "success")]
    message: String,
}

impl IntoResponse for BaseResp {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self).into_response()
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
    pub(crate) static ref JWT_SECRET: String = {
        let rng = rand::thread_rng();
        rng.sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    };
    pub(crate) static ref CIPHER: Cipher = Cipher::new();
}
