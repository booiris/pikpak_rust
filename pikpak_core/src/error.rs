use thiserror::Error;

use crate::api::ErrorResp;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Clone request failed: {0}")]
    CloneRequestError(String),
    #[error("Resp format error: {0}")]
    RespFormatError(#[from] serde_json::Error),
    #[error("Api error: {:?}", .0)]
    ApiError(ErrorResp),
    #[error("Auth error: {:?}", .0)]
    Oauth2Error(String),
    #[error("Request error: {:?}", .0)]
    RequestError(#[from] anyhow::Error),
}
