use core::downloader::Downloader;
use std::{
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use consts::*;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};
use rand::{distributions::Alphanumeric, Rng};
use reqwest::{header, Client};
use store::Store;

pub mod api;
mod consts;
pub mod core;
pub mod error;
pub mod extension;
mod store;
pub mod utils;

#[derive(Clone, Debug, Default)]
pub struct PkiPakApiConfig {
    pub cache_dir: Option<PathBuf>,
}

#[derive(Clone)]
pub struct PkiPakApiClient {
    pub(crate) inner: Arc<PkiPakApiClientInner>,
}

impl PkiPakApiClient {
    pub fn new(conf: Option<PkiPakApiConfig>, decrypt_key: String) -> Self {
        let h = Self {
            inner: Arc::new(PkiPakApiClientInner::new(conf, decrypt_key)),
        };
        if h.inner
            .downloader
            .set(Downloader::new(
                h.clone(),
                h.inner.store.pikpak_download_info.clone(),
            ))
            .is_err()
        {
            panic!("downloader has been initialized");
        }
        h
    }
}

pub(crate) struct PkiPakApiClientInner {
    pub client: Client,
    pub oauth2_client: BasicClient,
    pub device_id: String,
    pub store: Store,
    pub downloader: OnceLock<Downloader>,
}

impl PkiPakApiClientInner {
    pub fn new(conf: Option<PkiPakApiConfig>, decrypt_key: String) -> Self {
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
        headers.insert("Connection", header::HeaderValue::from_static("keep-alive"));

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("client build failed");

        let oauth2_client = BasicClient::new(
            ClientId::new(CLIENT_ID.into()),
            Some(ClientSecret::new(CLIENT_SECRET.into())),
            AuthUrl::new(AUTH_URL.into()).expect("parse auth url failed"),
            Some(TokenUrl::new(TOKEN_URL.into()).expect("parse token url failed")),
        )
        .set_auth_type(oauth2::AuthType::RequestBody);

        let store = Store::new(conf.and_then(|c| c.cache_dir), decrypt_key);

        Self {
            client,
            oauth2_client,
            device_id,
            store,
            downloader: OnceLock::new(),
        }
    }

    pub(crate) fn downloader(&self) -> &Downloader {
        self.downloader.get().unwrap()
    }
}

#[cfg(test)]
mod test {
    use std::sync::OnceLock;

    use dotenvy_macro::dotenv;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    use crate::api::Ident;

    #[ctor::ctor]
    fn init_test() {
        let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"));

        let formatting_layer = tracing_subscriber::fmt::layer()
            .pretty()
            .with_writer(std::io::stderr);

        tracing_subscriber::Registry::default()
            .with(env_filter)
            .with(tracing_error::ErrorLayer::default())
            .with(formatting_layer)
            .init();
    }

    pub fn test_client() -> &'static super::PkiPakApiClient {
        static CLIENT: OnceLock<super::PkiPakApiClient> = OnceLock::new();
        CLIENT.get_or_init(|| super::PkiPakApiClient::new(None, "1".into()))
    }

    #[cfg(feature = "__local_test")]
    pub fn test_ident() -> Ident {
        Ident {
            username: dotenv!("username").into(),
            password: dotenv!("password").into(),
        }
    }
}
