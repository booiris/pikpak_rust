use std::sync::Arc;

use consts::*;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{header, Client};

pub mod api;
mod consts;
pub mod core;
pub mod error;
pub mod extension;
mod utils;

#[derive(Clone, Debug, Default)]
pub struct PkiPakApiConfig {
    pub proxy: Option<String>,
}

#[derive(Clone)]
pub struct PkiPakApiClient {
    pub(crate) inner: Arc<PkiPakApiClientInner>,
}

impl PkiPakApiClient {
    pub fn new(conf: Option<PkiPakApiConfig>) -> Self {
        Self {
            inner: Arc::new(PkiPakApiClientInner::new(conf)),
        }
    }
}

pub(crate) struct PkiPakApiClientInner {
    pub client: Client,
    pub oauth2_client: BasicClient,
    pub device_id: String,
}

impl PkiPakApiClientInner {
    pub fn new(conf: Option<PkiPakApiConfig>) -> Self {
        let device_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let mut headers = header::HeaderMap::new();
        headers.insert("User-Agent", header::HeaderValue::from_static(USER_AGENT));
        headers.insert(
            "X-Device-Id",
            device_id.parse().expect("parse device id header failed"),
        );

        let mut client_builder: reqwest::ClientBuilder =
            reqwest::Client::builder().default_headers(headers);
        if let Some(proxy) = conf.and_then(|c| c.proxy) {
            client_builder =
                client_builder.proxy(reqwest::Proxy::all(proxy).expect("proxy format is invalid"));
        }
        let client = client_builder.build().expect("client build failed");

        let oauth2_client = BasicClient::new(
            ClientId::new(CLIENT_ID.into()),
            Some(ClientSecret::new(CLIENT_SECRET.into())),
            AuthUrl::new(AUTH_URL.into()).expect("parse auth url failed"),
            Some(TokenUrl::new(TOKEN_URL.into()).expect("parse token url failed")),
        );

        Self {
            client,
            oauth2_client,
            device_id,
        }
    }
}

#[cfg(test)]
mod test {
    use std::sync::OnceLock;

    use dotenv_codegen::dotenv;

    use crate::api::Ident;

    #[ctor::ctor]
    fn init_test() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .unwrap();
    }

    pub fn test_client() -> &'static super::PkiPakApiClient {
        static CLIENT: OnceLock<super::PkiPakApiClient> = OnceLock::new();
        CLIENT.get_or_init(|| super::PkiPakApiClient::new(None))
    }

    pub fn test_ident() -> Ident {
        Ident {
            username: dotenv!("username").into(),
            password: dotenv!("password").into(),
        }
    }
}
