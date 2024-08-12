use anyhow::Error;
use axum::Router;
use server::start_server;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let _log_file_guard = setup_server_logger();

    tokio::spawn(async {
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 22500));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        tracing::debug!("listening on {}", listener.local_addr().unwrap());

        let app = Router::new().nest_service("/", ServeDir::new("dist"));

        axum::serve(listener, app.layer(TraceLayer::new_for_http()))
            .await
            .unwrap();
    });

    start_server(
        "0.0.0.0",
        "22523",
        None,
        None,
        dotenv_codegen::dotenv!("decrypt_key").into(),
    )
    .await
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
