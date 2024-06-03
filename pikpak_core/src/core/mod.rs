use log::trace;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;

use crate::{
    api::{Ident, RespWrapper},
    error::Error,
    extension::api_option::{ApiOption, ApiSend},
    PkiPakApiClient,
};

pub mod download;
pub mod file;
pub mod folder;
pub mod token;

pub(crate) struct ApiClient<'c> {
    pub client: &'c PkiPakApiClient,
    pub ident: &'c Ident,
    pub option: &'c Option<ApiOption>,
}

impl<'c> PkiPakApiClient {
    pub(crate) fn api(&'c self, ident: &'c Ident, option: &'c Option<ApiOption>) -> ApiClient<'c> {
        ApiClient {
            client: self,
            ident,
            option,
        }
    }
}

impl ApiClient<'_> {
    pub(crate) async fn send_raw_req<Resp: DeserializeOwned>(
        &self,
        req: RequestBuilder,
    ) -> Result<Resp, Error> {
        let resp = req
            .api_send(self)
            .await?
            .text()
            .await
            .map_err(Error::NetworkError)?;

        trace!("response: {:#?}", resp);

        match serde_json::from_str::<RespWrapper<Resp>>(&resp) {
            Ok(RespWrapper::Success(resp)) => Ok(resp),
            Ok(RespWrapper::Err(err)) => Err(Error::ApiError(err)),
            Err(e) => Err(Error::RespFormatError(e)),
        }
    }

    #[inline]
    pub(crate) fn http_client(&self) -> &reqwest::Client {
        &self.client.inner.client
    }
}
