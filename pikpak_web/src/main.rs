use std::env;

use anyhow::Error;
use axum::Router;
use pikpak_web::start_server;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _log_file_guard = setup_server_logger();

    let host = env::var("PIKPAK_WEB_HOST").unwrap_or("0.0.0.0".to_string());
    let port = env::var("PIKPAK_WEB_PORT").unwrap_or("22523".to_string());
    let cache_dir = env::var("PIKPAK_WEB_CACHE_DIR")
        .ok()
        .map(|s| s.into())
        .or(None);
    let decrypt_key = env::var("PIKPAK_WEB_DECRYPT_KEY")
        .expect("missing decrypt key, set PIKPAK_WEB_DECRYPT_KEY");

    let frontend_host = env::var("PIKPAK_WEB_FRONTEND_HOST").unwrap_or("0.0.0.0".to_string());
    let frontend_port = env::var("PIKPAK_WEB_FRONTEND_PORT").unwrap_or("22500".to_string());
    let frontend_addr = format!("{}:{}", frontend_host, frontend_port);
    let listener = tokio::net::TcpListener::bind(&frontend_addr).await.unwrap();

    tokio::spawn(async {
        let app = Router::new().nest_service("/", ServeDir::new("dist"));

        tracing::info!("front end listening on {}", listener.local_addr().unwrap());

        axum::serve(listener, app.layer(TraceLayer::new_for_http()))
            .await
            .unwrap();
    });

    start_server(host, port, cache_dir, decrypt_key).await
}

fn setup_server_logger() -> WorkerGuard {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_writer(std::io::stderr);

    let file_appender = tracing_appender::rolling::never("logs", "app.log");
    let (non_blocking_appender, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking_appender);

    tracing_subscriber::Registry::default()
        .with(env_filter)
        .with(tracing_error::ErrorLayer::default())
        .with(formatting_layer)
        .with(file_layer)
        .init();
    color_eyre::install().expect("install color eyre");
    guard
}
