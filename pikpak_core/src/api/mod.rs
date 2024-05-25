use log::trace;
use reqwest::RequestBuilder;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
    error::Error,
    extension::api_option::{ApiOption, ApiSend},
    PkiPakApiClient,
};

pub mod login;
pub(crate) mod token;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ErrorResp {
    pub error: String,
    pub error_code: i64,
    pub error_description: String,
    pub details: Vec<Detail>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Detail {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub reason: Option<String>,
    pub locale: Option<String>,
    pub message: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum RespWrapper<T> {
    Ok(T),
    Err(ErrorResp),
}

#[derive(Default, PartialEq, Eq, Hash, Clone)]
pub struct Ident {
    pub username: String,
    pub password: String,
}

impl PkiPakApiClient {
    pub(crate) async fn send_raw_req<Resp: DeserializeOwned>(
        &self,
        mut req: RequestBuilder,
        ident: &Ident,
        option: Option<ApiOption>,
    ) -> Result<Resp, Error> {
        let token = self.auth_token(ident).await?;
        req = req.bearer_auth(token);
        let resp = req
            .api_send(option.unwrap_or_default())
            .await?
            .text()
            .await
            .map_err(Error::NetworkError)?;

        trace!("response: {:#?}", resp);

        match serde_json::from_str::<RespWrapper<Resp>>(&resp) {
            Ok(RespWrapper::Ok(resp)) => Ok(resp),
            Ok(RespWrapper::Err(err)) => Err(Error::ApiError(err)),
            Err(e) => Err(Error::RespFormatError(e)),
        }
    }
}
