use anyhow::{Context, Error};
use axum::{
    http::Method,
    routing::{get, post},
    serve, Router,
};
use log::{error, info};
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{handlers::login::login, openapi_doc::ApiDoc};

mod handlers;
mod openapi_doc;
mod utils;

pub async fn start_server(host: impl Into<String>, port: impl Into<&str>) -> Result<(), Error> {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new().route("/", get(hello)).nest(
        "/api",
        Router::new()
            .route("/login", post(login))
            .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .layer(cors),
    );

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
