use anyhow::Error;
use fern::colors::Color;
use server::start_server;

#[tokio::main]
async fn main() -> Result<(), Error> {
    setup_server_logger(log::LevelFilter::Debug)?;
    start_server("0.0.0.0", "22522", None, None).await
}

fn setup_server_logger(level: log::LevelFilter) -> Result<(), anyhow::Error> {
    let colors = fern::colors::ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Blue)
        .debug(Color::White)
        .trace(Color::BrightBlack);
    let logger = fern::Dispatch::new()
        .level(level)
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {} {} {}] {}",
                humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                colors.color(record.level()),
                record.target(),
                record.line().unwrap_or_default(),
                message
            ))
        })
        .chain(std::io::stdout());

    Ok(logger.apply()?)
}
