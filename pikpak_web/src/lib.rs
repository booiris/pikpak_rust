use std::path::PathBuf;

use anyhow::{anyhow, Context, Error};
use axum::{
    http::Method,
    routing::{get, post},
    serve, Router,
};

use handlers::{
    download_pause::download_pause, download_remove::download_remove,
    download_resume::download_resume, mget_download_status::mget_download_status,
};
use pikpak_core::{PkiPakApiClient, PkiPakApiConfig};
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::{Any, CorsLayer},
};
use tracing::{error, info};

#[cfg(feature = "utoipa")]
use crate::handlers::ApiDoc;
#[cfg(feature = "utoipa")]
use utoipa::OpenApi;

#[cfg(feature = "utoipa")]
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{
    download_begin::download_begin, login::login, remote_list::remote_list, PIKPAK_CORE_CLIENT,
};

mod extension;
mod handlers;
mod utils;

pub async fn start_server(
    host: impl Into<String>,
    port: impl AsRef<str>,
    cache_dir: Option<PathBuf>,
    decrypt_key: String,
) -> Result<(), Error> {
    info!("[rust pikpak server] starting server");

    PIKPAK_CORE_CLIENT
        .set(PkiPakApiClient::new(
            Some(PkiPakApiConfig { cache_dir }),
            decrypt_key,
        ))
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
                .route("/download_resume", post(download_resume))
                .route("/download_pause", post(download_pause))
                .route("/download_remove", post(download_remove))
                .route("/mget_download_status", post(mget_download_status)),
        )
        .layer(cors)
        .layer(CatchPanicLayer::new());

    #[cfg(feature = "utoipa")]
    let app = app.merge(SwaggerUi::new("/doc").url("/openapi.json", ApiDoc::openapi()));

    let addr = host.into() + ":" + port.as_ref();
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
