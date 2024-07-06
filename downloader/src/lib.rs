pub mod chunk;
pub mod downloader;
pub mod error;
pub mod limiter;
pub mod status_tracker;
pub mod worker;

pub use downloader::Downloader;

#[cfg(test)]
mod test {
    #[ctor::ctor]
    fn init_test() {
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .try_init()
            .unwrap();
    }
}
