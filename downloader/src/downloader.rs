use std::collections::VecDeque;
use std::{io::SeekFrom, path::PathBuf, sync::Arc};

use crate::error::Error::ThreadError;
use crate::{error::Error::FileWriteError, worker::TaskResultState};
use bytes::Buf;
use futures_util::future::BoxFuture;
use futures_util::FutureExt;
use parking_lot::Mutex;
use reqwest::Client;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;

use crate::{
    chunk::ChunkRange,
    error::Error,
    limiter::SpeedLimiter,
    status_tracker::{Status, StatusTracker},
    worker::{do_work, WorkItem},
};

pub struct Downloader {
    client: Client,
}

pub struct DownloadConfig {
    pub url: String,
    pub download_path: PathBuf,
    pub chunk_size: Option<usize>,
    pub worker_num: Option<usize>,
    pub speed_limit_per_sec: Option<usize>,
    pub total_size: u64,
}

impl Downloader {
    pub fn new(client: Client) -> Downloader {
        Downloader { client }
    }

    pub fn create_runner(&self, conf: DownloadConfig) -> DownloadManger {
        let done = CancellationToken::new();
        let inner = DownloadManagerInner {
            url: conf.url,
            download_path: Arc::new(conf.download_path.clone()),
            speed_limiter: SpeedLimiter::new(conf.speed_limit_per_sec),
            status_tracker: StatusTracker::new(conf.download_path, done.clone()),
            client: self.client.clone(),
            worker_num: Mutex::new(conf.worker_num.unwrap_or(8)),
            chunk_size: Mutex::new(conf.chunk_size.unwrap_or(2 * 1024 * 1024)),
            runner_lock: tokio::sync::Mutex::new(()),
            done,
            total_size: conf.total_size,
        };
        DownloadManger(Arc::new(inner))
    }
}

#[derive(Clone)]
pub struct DownloadManger(Arc<DownloadManagerInner>);

pub(crate) struct DownloadManagerInner {
    url: String,
    download_path: Arc<PathBuf>,
    speed_limiter: SpeedLimiter,
    status_tracker: StatusTracker,
    client: Client,
    worker_num: Mutex<usize>,
    chunk_size: Mutex<usize>,
    runner_lock: tokio::sync::Mutex<()>,
    done: CancellationToken,
    total_size: u64,
}

impl Drop for DownloadManagerInner {
    fn drop(&mut self) {
        self.done.cancel();
    }
}

impl DownloadManger {
    /// Change the download speed limit. if `limit` is `None`, the speed limit will be removed.
    pub async fn change_limit(&mut self, limit: Option<usize>) {
        self.0.speed_limiter.change(limit).await;
    }

    pub async fn change_worker_num(&self, worker_num: usize) {
        if worker_num == 0 {
            return;
        }
        *self.0.worker_num.lock() = worker_num;
    }

    pub async fn change_chunk_size(&self, chunk_size: usize) {
        if chunk_size == 0 {
            return;
        }
        *self.0.chunk_size.lock() = chunk_size;
    }

    pub async fn run(
        &self,
    ) -> Result<(BoxFuture<'static, Result<(), Error>>, CancellationToken), Error> {
        let self_clone = self.clone();
        let cancel = CancellationToken::new();
        let return_cancel = cancel.clone();
        let mut writer = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self_clone.0.download_path.as_ref())
            .await
            .map_err(FileWriteError)?;

        let runner = async move {
            let _lock = self_clone.0.runner_lock.lock().await;
            let client = self_clone.0.client.clone();
            let speed_limiter = self_clone.0.speed_limiter.clone();

            let status = self_clone.0.status_tracker.load_status_from_file().await;
            if let Err(e) = writer.seek(SeekFrom::Start(status.downloaded)).await {
                log::error!("[download runner] seek file error: {:?}", e);
                return Err(FileWriteError(e));
            }

            let mut working_tasks = VecDeque::new();
            let mut downloading_now = status.downloaded;
            loop {
                for _ in 0..*self_clone.0.worker_num.lock() - working_tasks.len() {
                    if downloading_now >= self_clone.0.total_size {
                        break;
                    }
                    let mut len = *self_clone.0.chunk_size.lock() as u64;
                    if downloading_now + len > self_clone.0.total_size {
                        len = self_clone.0.total_size - downloading_now + 1;
                    }
                    let task = ChunkRange::from_len(downloading_now, len);
                    downloading_now += len;

                    log::debug!("tasks: {:?} downloading_now: {}", task, downloading_now);

                    let req = reqwest::Request::new(
                        reqwest::Method::GET,
                        self_clone.0.url.parse().unwrap(),
                    );

                    let work_item = WorkItem {
                        task,
                        cancel: cancel.clone(),
                        path: self_clone.0.download_path.clone(),
                        request: req,
                    };
                    let client = client.clone();
                    let speed_limiter = speed_limiter.clone();

                    working_tasks.push_back(tokio::spawn(async move {
                        do_work(client, work_item, speed_limiter.clone()).await
                    }));
                }

                if let Some(downloaded_task) = working_tasks.pop_front() {
                    let mut res = downloaded_task.await.map_err(|e| {
                        log::error!("[download runner thread error] {:?}", e);
                        ThreadError(e)
                    })??;

                    log::debug!("working_tasks: {}", res.task);

                    while res.data.has_remaining() {
                        writer.write_buf(&mut res.data).await.map_err(|e| {
                            log::error!("[download runner] write file error: {:?}", e);
                            FileWriteError(e)
                        })?;
                    }
                    match res.state {
                        TaskResultState::Done => {
                            log::debug!("working_tasks done: {}", res.task);
                        }
                        TaskResultState::Canceled => {
                            log::info!("[download runner] task canceled, task: {:?}", res.task);
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
            Ok(())
        };

        Ok((runner.boxed(), return_cancel))
    }

    // pub async fn stop(&self, remove_file: bool) {
    //     self.0.cancel.cancel();

    //     todo!()
    // }

    pub async fn get_status(&self) -> Status {
        self.0.status_tracker.get_status().await
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_downloader() {
        let client = reqwest::Client::new();
        let downloader = Downloader::new(client);
        let runner = downloader.create_runner(DownloadConfig {
            url: "https://speed.cloudflare.com/robots.txt".to_string(),
            download_path: PathBuf::from("cache/test"),
            chunk_size: Some(2),
            worker_num: None,
            speed_limit_per_sec: None,
            total_size: 22,
        });
        let (runner, _cancel) = runner.run().await.unwrap();
        runner.await.unwrap();
    }

    #[tokio::test]
    async fn test_downloader_failed() {
        let client = reqwest::Client::new();
        let downloader = Downloader::new(client);
        let runner = downloader.create_runner(DownloadConfig {
            url: "https://speed.cloudflare.com/__down?during=download&bytes=104857600".to_string(),
            download_path: PathBuf::from("cache/test"),
            chunk_size: None,
            worker_num: None,
            speed_limit_per_sec: None,
            total_size: 22,
        });
        let (runner, _cancel) = runner.run().await.unwrap();
        assert!(runner.await.is_err());
    }
}
