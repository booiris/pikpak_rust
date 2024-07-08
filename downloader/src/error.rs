use thiserror::Error;
use tokio::io;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("FileWrite error: {0}")]
    FileWriteError(#[from] io::Error),
    #[error("Clone request failed: {0}")]
    CloneRequestError(String),
    #[error("Request error: {:?}", .0)]
    RequestError(#[from] anyhow::Error),
    #[error("Thread error: {0}")]
    ThreadError(#[from] tokio::task::JoinError),
}
