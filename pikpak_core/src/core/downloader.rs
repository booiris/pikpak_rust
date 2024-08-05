use std::{
    path::PathBuf,
    sync::{atomic::AtomicU64, Arc},
    time::Duration,
};

use ahash::AHashMap;
use anyhow::Context;
use atomic_float::AtomicF64;
use futures_util::StreamExt;
use humansize::DECIMAL;
use log::{info, warn};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering::*;
use tokio::{
    fs::{File, OpenOptions},
    select,
    sync::Semaphore,
};
use tokio_util::sync::CancellationToken;

use crate::{
    api::Ident,
    error::Error,
    store::{download_status_store::PikPakDownloadInfo, UserName},
    PkiPakApiClient, USER_AGENT,
};

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct InnerDownloadStatus {
    pub total: u64,
    pub downloaded_time: Arc<Mutex<Duration>>,
    pub downloaded: Arc<AtomicU64>,

    pub file_id: String,
    pub remote_file_name: String,
    pub download_to_local_path: PathBuf,
    pub status: Arc<RwLock<DownloadStatusKind>>,
    pub id: Ident,

    // do not store
    #[serde(skip)]
    pub current_speed: Arc<AtomicF64>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DownloadStatus {
    pub total: u64,
    pub downloaded: u64,
    pub current_speed: f64,
    pub downloaded_time: Duration,

    pub file_id: String,
    pub remote_file_name: String,
    pub download_to_local_path: PathBuf,
    pub status: DownloadStatusKind,

    #[serde(skip)]
    pub id: Ident,
}

impl InnerDownloadStatus {
    fn display(&self) -> DownloadStatus {
        DownloadStatus {
            total: self.total,
            downloaded: self.downloaded.load(std::sync::atomic::Ordering::Relaxed),
            current_speed: self
                .current_speed
                .load(std::sync::atomic::Ordering::Relaxed),
            downloaded_time: *self.downloaded_time.lock(),
            file_id: self.file_id.clone(),
            remote_file_name: self.remote_file_name.clone(),
            download_to_local_path: self.download_to_local_path.clone(),
            status: self.status.read().clone(),
            id: self.id.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DownloadStatusKind {
    Downloading,
    Paused,
    Completed,
    Waiting,
    HasError(String),
}

impl PartialEq for DownloadStatusKind {
    fn eq(&self, _other: &Self) -> bool {
        matches!(self, _other)
    }
}

pub type FileID = String;

pub struct Downloader {
    client: PkiPakApiClient,

    file_status: PikPakDownloadInfo,
    runner_control: Mutex<AHashMap<FileID, CancellationToken>>,
    semaphore: Arc<RwLock<AHashMap<UserName, Arc<Semaphore>>>>,
}

impl Drop for Downloader {
    fn drop(&mut self) {
        for (_, cancel) in self.runner_control.lock().drain() {
            cancel.cancel();
        }
    }
}

impl Downloader {
    pub(crate) fn new(client: PkiPakApiClient, file_status: PikPakDownloadInfo) -> Self {
        let h = Self {
            client,
            file_status,
            runner_control: Mutex::new(AHashMap::new()),
            semaphore: Arc::new(RwLock::new(AHashMap::new())),
        };

        let file_status = h.file_status.get_all();

        for (user, info_map) in &file_status {
            let download_info = info_map.read();
            let download_info = download_info.download_info.read();

            h.semaphore
                .write()
                .insert(user.clone(), Arc::new(Semaphore::new(4)));

            for (id, status) in download_info
                .iter()
                .filter(|(_, status)| *status.status.read() == DownloadStatusKind::Downloading)
            {
                let cancel = CancellationToken::new();
                h.runner_control.lock().insert(id.clone(), cancel.clone());
                h.run_downloader(status, cancel);
            }

            for (id, status) in download_info
                .iter()
                .filter(|(_, status)| *status.status.read() == DownloadStatusKind::Waiting)
            {
                let cancel = CancellationToken::new();
                h.runner_control.lock().insert(id.clone(), cancel.clone());
                h.run_downloader(status, cancel);
            }
        }

        h
    }

    pub fn get_status_by_id(&self, file_id: &FileID, id: &Ident) -> Option<DownloadStatus> {
        self.file_status
            .get(&id.username)
            .read()
            .download_info
            .read()
            .get(file_id)
            .map(|x| x.display())
    }

    pub fn get_status_by_filter(
        &self,
        filter: &Option<Vec<DownloadStatusKind>>,
        id: &Ident,
    ) -> AHashMap<FileID, DownloadStatus> {
        let binding = self.file_status.get(&id.username);
        let binding = binding.read();
        let file_status = binding.download_info.read();
        if let Some(filter) = filter {
            file_status
                .iter()
                .filter(|(_, status)| filter.contains(&status.status.read()))
                .map(|(file_id, status)| (file_id.clone(), status.display()))
                .collect()
        } else {
            file_status
                .iter()
                .map(|(file_id, status)| (file_id.clone(), status.display()))
                .collect()
        }
    }

    pub fn start_download(
        &self,
        file_id: FileID,
        download_to_local_path: PathBuf,
        id: &Ident,
        remote_file_name: String,
        remote_file_size: u64,
    ) -> Result<(), Error> {
        if self.runner_control.lock().contains_key(&file_id) {
            return Ok(());
        }
        if self
            .file_status
            .get(&id.username)
            .read()
            .download_info
            .read()
            .get(&file_id)
            .map(|x| {
                *x.status.read() == DownloadStatusKind::Downloading
                    || *x.status.read() == DownloadStatusKind::Waiting
                    || *x.status.read() == DownloadStatusKind::Completed
            })
            == Some(true)
        {
            return Ok(());
        }

        if download_to_local_path.exists() {
            return Err(Error::RequestError(anyhow::anyhow!(
                "file already exists, path: {}",
                download_to_local_path.display()
            )));
        }

        let cancel = CancellationToken::new();

        self.runner_control
            .lock()
            .insert(file_id.clone(), cancel.clone());

        let download_status = InnerDownloadStatus {
            total: remote_file_size,
            downloaded: Arc::new(AtomicU64::new(0)),
            current_speed: Arc::new(AtomicF64::new(0.0)),
            downloaded_time: Arc::new(Mutex::new(Duration::from_secs(0))),
            file_id: file_id.clone(),
            remote_file_name,
            download_to_local_path: download_to_local_path.clone(),
            status: Arc::new(RwLock::new(DownloadStatusKind::Waiting)),
            id: id.clone(),
        };

        self.run_downloader(&download_status, cancel);

        self.file_status
            .get(&id.username)
            .read()
            .download_info
            .write()
            .insert(file_id.clone(), download_status);

        Ok(())
    }

    fn run_downloader(&self, download_status: &InnerDownloadStatus, cancel: CancellationToken) {
        let downloaded = download_status.downloaded.clone();
        let current_speed = download_status.current_speed.clone();
        let downloaded_time = download_status.downloaded_time.clone();
        let status = download_status.status.clone();
        let download_to_local_path = download_status.download_to_local_path.clone();
        let client = self.client.clone();
        let id = download_status.id.clone();
        let file_id = download_status.file_id.clone();

        let semaphore = self
            .semaphore
            .write()
            .entry(id.username.clone())
            .or_insert(Arc::new(Semaphore::new(4)))
            .clone();

        tokio::spawn(async move {
            let _permit = match semaphore.acquire().await {
                Ok(x) => x,
                Err(e) => {
                    log::error!("[download] semaphore acquire error: {:?}", e);
                    *status.write() = DownloadStatusKind::HasError(format!("{:?}", e));
                    return;
                }
            };

            let mut out_file = match OpenOptions::new()
                .create(true)
                .append(true)
                .mode(0o644)
                .open(&download_to_local_path)
                .await
            {
                Ok(x) => x,
                Err(e) => {
                    log::error!("[download] open file failed, error: {:?}", e);
                    *status.write() = DownloadStatusKind::HasError(format!("{:?}", e));
                    return;
                }
            };

            *status.write() = DownloadStatusKind::Downloading;

            let out_cancel = cancel.clone();

            loop {
                let info = match client.api(&id, &None).get_file_by_id(&file_id).await {
                    Ok(x) => x,
                    Err(e) => {
                        log::error!("[download] get url failed, error: {:?}", e);
                        *status.write() = DownloadStatusKind::HasError(format!("{:?}", e));
                        return;
                    }
                };

                let p = DownloadUrlParam {
                    client: &client,
                    url: &info.links.application_octet_stream.url,
                    path: &download_to_local_path,
                    out_file: &mut out_file,
                    downloaded: &downloaded,
                    current_speed: &current_speed,
                    downloaded_time: &downloaded_time,
                    cancel: cancel.clone(),
                };

                select! {
                    _ = out_cancel.cancelled() => {
                        *status.write() = DownloadStatusKind::Paused;
                        break;
                    }
                    res = download_url(
                        p
                    ) => {
                        match res{
                            Ok(()) => {
                                *status.write() = DownloadStatusKind::Completed;
                                break;
                            }
                            Err(e) => {
                                log::error!("[download] download failed, error: {:?}", e);

                                if matches!(e, DownloadError::ResumeFailed) {
                                    *status.write() = DownloadStatusKind::HasError("resume failed".to_string());
                                    break;
                                }

                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn pause_download(&self, file_id: &FileID, id: &Ident) {
        if let Some(x) = self.runner_control.lock().remove(file_id) {
            x.cancel()
        }
        if let Some(x) = self
            .file_status
            .get(&id.username)
            .read()
            .download_info
            .write()
            .get_mut(file_id)
        {
            if *x.status.read() == DownloadStatusKind::Downloading {
                *x.status.write() = DownloadStatusKind::Paused;
            }
        }
    }

    pub fn remove_download(&self, file_id: &FileID, id: &Ident) {
        if let Some(x) = self.runner_control.lock().remove(file_id) {
            x.cancel()
        }
        self.file_status
            .get(&id.username)
            .read()
            .download_info
            .write()
            .remove(file_id);
    }
}

#[derive(Debug)]
enum DownloadError {
    ResumeFailed,
    #[allow(dead_code)]
    DownloadError(String),
}

struct DownloadUrlParam<'a> {
    client: &'a PkiPakApiClient,
    url: &'a String,
    path: &'a PathBuf,
    out_file: &'a mut File,
    downloaded: &'a AtomicU64,
    current_speed: &'a AtomicF64,
    downloaded_time: &'a Mutex<Duration>,
    cancel: CancellationToken,
}

async fn download_url(p: DownloadUrlParam<'_>) -> Result<(), DownloadError> {
    let size = p.out_file.metadata().await.unwrap().len();
    p.downloaded.store(size, SeqCst);
    let mut req = p
        .client
        .inner
        .client
        .get(p.url)
        .header("User-Agent", USER_AGENT);
    let resume = size != 0;
    if resume {
        info!(
            "resuming from {} bytes, file: {:?}",
            humansize::format_size(size, DECIMAL),
            p.path
        );
        req = req.header("Range", format!("bytes={}-", size));
    }
    let resp = req
        .send()
        .await
        .context("download url failed")
        .map_err(|e| DownloadError::DownloadError(format!("{:?}", e)))?;

    if resume && resp.status() != 206 {
        warn!("resume failed, status: {}", resp.status());
        return Err(DownloadError::ResumeFailed);
    }

    let mut stream = resp.bytes_stream();

    loop {
        let time_start = tokio::time::Instant::now();
        select! {
            _ = p.cancel.cancelled() => {
                return Ok(());
            }
            item = stream.next() => {
                match item {
                    Some(item) => {
                        if let Ok(item) = item {
                            let copy_cnt = tokio::io::copy(&mut item.as_ref(), p.out_file).await.context("copy error").map_err(|e| DownloadError::DownloadError(format!("{:?}", e)))?;
                            let time_end = tokio::time::Instant::now();
                            let time_cost = time_end - time_start;
                            *p.downloaded_time.lock() += time_cost;
                            p.current_speed.store(copy_cnt as f64 / time_cost.as_secs_f64(), std::sync::atomic::Ordering::Relaxed);
                            p.downloaded.fetch_add(copy_cnt, Relaxed);
                        }
                    },
                    None => {
                        break;
                    }
                }

            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use dotenv_codegen::dotenv;

    use super::*;
    use crate::test::{test_client, test_ident};
    use std::path::PathBuf;

    #[cfg(feature = "__local_test")]
    #[tokio::test]
    async fn test_download() {
        let client = test_client();
        let ident = test_ident();

        let api = client.api(&ident, &None);

        let file = api
            .get_file_by_id(dotenv!("download_test_file_id"))
            .await
            .expect("get file id error");

        let download_to_local_path = PathBuf::from("cache/test.mp4");

        let downloader = client.inner.downloader();
        let file_id = file.id;

        downloader
            .start_download(
                file_id.clone(),
                download_to_local_path.clone(),
                &ident,
                file.name,
                file.size.parse().unwrap(),
            )
            .unwrap();

        tokio::time::sleep(Duration::from_secs(10)).await;

        let status = downloader.get_status_by_id(&file_id, &ident).unwrap();
        if status.status != DownloadStatusKind::Downloading
            && status.status != DownloadStatusKind::Completed
        {
            panic!("download status invalid");
        }

        downloader.pause_download(&file_id, &ident);

        tokio::time::sleep(Duration::from_secs(1)).await;

        let status = downloader.get_status_by_id(&file_id, &ident).unwrap();
        if status.status != DownloadStatusKind::Paused
            && status.status != DownloadStatusKind::Completed
        {
            panic!("pause status invalid");
        }

        downloader.remove_download(&file_id, &ident);

        tokio::time::sleep(Duration::from_secs(1)).await;

        let status = downloader.get_status_by_id(&file_id, &ident);

        if status.is_some() {
            panic!("status should be None");
        }
    }
}
