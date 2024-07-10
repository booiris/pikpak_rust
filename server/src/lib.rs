use std::path::PathBuf;

use anyhow::{anyhow, Context, Error};
use axum::{
    http::Method,
    routing::{get, post},
    serve, Router,
};

use handlers::{
    download_pause::download_pause, download_remove::download_remove,
    mget_download_status::mget_download_status,
};
use log::{error, info};
use pikpak_core::{PkiPakApiClient, PkiPakApiConfig};
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::{Any, CorsLayer},
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{
    download_begin::download_begin, login::login, remote_list::remote_list, ApiDoc,
    PIKPAK_CORE_CLIENT,
};

mod extension;
mod handlers;
mod utils;

pub async fn start_server(
    host: impl Into<String>,
    port: impl Into<&str>,
    proxy: Option<String>,
    cache_dir: Option<PathBuf>,
) -> Result<(), Error> {
    PIKPAK_CORE_CLIENT
        .set(PkiPakApiClient::new(Some(PkiPakApiConfig {
            proxy,
            cache_dir,
        })))
        .map_err(|_| {
            error!("[rust pikpak server] set pikpak core client error");
            anyhow!("set pikpak core client error")
        })?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(hello))
        .nest(
            "/api",
            Router::new()
                .route("/login", post(login))
                .route("/remote_list", get(remote_list))
                .route("/download_begin", post(download_begin))
                .route("/download_pause", post(download_pause))
                .route("/download_remove", post(download_remove))
                .route("/mget_download_status", get(mget_download_status)),
        )
        .merge(SwaggerUi::new("/doc").url("/openapi.json", ApiDoc::openapi()))
        .layer(cors)
        .layer(CatchPanicLayer::new());

    let addr = host.into() + ":" + port.into();
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| {
            error!("[rust pikpak server] create listener error: {}", e);
            e
        })
        .context("init listener error")?;

    info!("[rust pikpak server] Server listening on {}", addr);

    if let Err(err) = serve(listener, app).await {
        error!("[rust pikpak server] Server error: {}", err);
        Err(err).context("server error")
    } else {
        Ok(())
    }
}

async fn hello() -> &'static str {
    "Hello, this is rust pikpak backend!"
}
