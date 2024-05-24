use anyhow::Error;
use server::start_server;

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_server_logger(log::LevelFilter::Debug)?;
    start_server("0.0.0.0", "22522").await
}

fn setup_server_logger(level: log::LevelFilter) -> Result<(), anyhow::Error> {
    let logger = fern::Dispatch::new()
        .level(level)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {} {}] {}",
                humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                record.level(),
                record.target(),
                record.line().unwrap_or_default(),
                message
            ))
        })
        .chain(std::io::stdout());

    Ok(logger.apply()?)
}
