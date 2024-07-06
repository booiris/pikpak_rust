use std::{io::SeekFrom, path::PathBuf, sync::Arc};

use crate::{error::Error::FileWriteError, worker::TaskResultState};
use bytes::Buf;
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
}

impl Downloader {
    pub fn new(client: Client) -> Downloader {
        Downloader { client }
    }

    pub fn create_runner(&self, conf: DownloadConfig) -> DownloadRunner {
        let done = CancellationToken::new();
        let inner = DownloadRunnerInner {
            url: conf.url,
            download_path: Arc::new(conf.download_path.clone()),
            speed_limiter: SpeedLimiter::new(conf.speed_limit_per_sec),
            status_tracker: StatusTracker::new(conf.download_path, done.clone()),
            client: self.client.clone(),
            worker_num: Mutex::new(conf.worker_num.unwrap_or(8)),
            chunk_size: Mutex::new(conf.chunk_size.unwrap_or(2 * 1024 * 1024)),
            runner_lock: tokio::sync::Mutex::new(()),
            done,
        };
        DownloadRunner(Arc::new(inner))
    }
}

#[derive(Clone)]
pub struct DownloadRunner(Arc<DownloadRunnerInner>);

pub(crate) struct DownloadRunnerInner {
    url: String,
    download_path: Arc<PathBuf>,
    speed_limiter: SpeedLimiter,
    status_tracker: StatusTracker,
    client: Client,
    worker_num: Mutex<usize>,
    chunk_size: Mutex<usize>,
    runner_lock: tokio::sync::Mutex<()>,
    done: CancellationToken,
}

impl Drop for DownloadRunnerInner {
    fn drop(&mut self) {
        self.done.cancel();
    }
}

impl DownloadRunner {
    /// Change the download speed limit. if `limit` is `None`, the speed limit will be removed.
    pub async fn change_limit(&mut self, limit: Option<usize>) {
        self.0.speed_limiter.change(limit).await;
    }

    pub async fn change_worker_num(&self, worker_num: usize) {
        *self.0.worker_num.lock() = worker_num;
    }

    pub async fn change_chunk_size(&self, chunk_size: usize) {
        *self.0.chunk_size.lock() = chunk_size;
    }

    pub async fn run(&self) -> Result<CancellationToken, Error> {
        let self_clone = self.clone();
        let cancel = CancellationToken::new();
        let return_cancel = cancel.clone();
        let mut writer = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self_clone.0.download_path.as_ref())
            .await
            .map_err(FileWriteError)?;

        tokio::spawn(async move {
            let _lock = self_clone.0.runner_lock.lock().await;
            let client = self_clone.0.client.clone();
            let speed_limiter = self_clone.0.speed_limiter.clone();

            let status = self_clone.0.status_tracker.load_status_from_file().await;
            if let Err(e) = writer.seek(SeekFrom::Start(status.downloaded)).await {
                log::error!("[download runner] seek file error: {:?}", e);
                return;
            }

            let mut working_tasks = Vec::new();
            let mut downloading_now = status.downloaded;
            loop {
                for _ in 0..*self_clone.0.worker_num.lock() - working_tasks.len() {
                    if downloading_now >= status.total {
                        break;
                    }
                    let mut len = *self_clone.0.chunk_size.lock() as u64;
                    if downloading_now + len > status.total {
                        len = status.total - downloading_now + 1;
                    }
                    let tasks = ChunkRange::from_len(downloading_now, len);
                    downloading_now += len;

                    let req = reqwest::Request::new(
                        reqwest::Method::GET,
                        self_clone.0.url.parse().unwrap(),
                    );

                    let work_item = WorkItem {
                        task: tasks,
                        cancel: cancel.clone(),
                        path: self_clone.0.download_path.clone(),
                        request: req,
                    };
                    let client = client.clone();
                    let speed_limiter = speed_limiter.clone();

                    working_tasks.push(tokio::spawn(async move {
                        do_work(client, work_item, speed_limiter.clone()).await
                    }));
                }

                if let Some(downloaded_task) = working_tasks.pop() {
                    let res = downloaded_task.await.map_err(|e| {
                        log::error!("[download runner thread error] {:?}", e);
                        e
                    });
                    if res.is_err() {
                        break;
                    }
                    let res = res.unwrap();
                    match res {
                        Ok(mut res) => {
                            while res.data.has_remaining() {
                                if let Err(e) = writer.write_buf(&mut res.data).await {
                                    log::error!("[download runner] write file error: {:?}", e);
                                    break;
                                }
                            }
                            match res.state {
                                TaskResultState::Done => {}
                                TaskResultState::Canceled => {
                                    log::info!(
                                        "[download runner] task canceled, task: {:?}",
                                        res.task
                                    );
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("[download runner] {:?}", e);
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        });
        Ok(return_cancel)
    }

    // pub async fn stop(&self, remove_file: bool) {
    //     self.0.cancel.cancel();

    //     todo!()
    // }

    pub async fn get_status(&self) -> Status {
        self.0.status_tracker.get_status().await
    }
}
