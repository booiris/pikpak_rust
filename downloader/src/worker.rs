use std::{path::PathBuf, sync::Arc};

use anyhow::anyhow;
use bytes::{Bytes, BytesMut};
use futures_util::StreamExt;
use headers::HeaderMapExt;
use reqwest::{Client, Request};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{chunk::ChunkRange, error::Error, limiter::SpeedLimiter};

pub enum TaskResultState {
    Done,
    Canceled,
}

pub struct WorkItem {
    pub task: ChunkRange,
    pub cancel: CancellationToken,
    pub path: Arc<PathBuf>,
    pub request: Request,
}

pub struct WorkResult {
    pub state: TaskResultState,
    pub data: Bytes,
    pub task: ChunkRange,
}

pub async fn do_work(
    client: Client,
    work_item: WorkItem,
    limiter: SpeedLimiter,
) -> Result<WorkResult, Error> {
    let mut chunk_bytes = BytesMut::with_capacity(work_item.task.len() as usize);
    let mut req = work_item.request;

    req.headers_mut()
        .typed_insert(ChunkRange::new(work_item.task.start, work_item.task.end).to_range_header());

    log::debug!("req: {:#?}", req);

    let mut stream = None;
    let url = req.url().clone();
    for _ in 0..5 {
        let req = req.try_clone().unwrap();
        let temp_stream = client.execute(req).await.map_err(|e| {
            log::error!(
                "[do work] request failed, err: {}, url: {}, range: {}, path: {:?}",
                e,
                url,
                work_item.task,
                work_item.path
            );
            e
        })?;

        if temp_stream.headers().get("Content-Range").is_none() {
            log::error!(
                "[do work] request failed, no content-range header resp, url: {}, range: {}, path: {:?}, resp header: {:?}",
                url,
                work_item.task,
                work_item.path,
                temp_stream.headers()
            );
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            continue;
        } else {
            stream = Some(temp_stream);
            break;
        }
    }

    if stream.is_none() {
        return Err(anyhow!("no valid response").into());
    }

    let mut stream = stream.unwrap().bytes_stream();

    let download_future = async {
        while let Some(bytes) = stream.next().await {
            let bytes = bytes.map_err(|e| {
                log::error!(
                    "[do work] request stream failed, err: {}, url: {}, range: {}, path: {:?}",
                    e,
                    url,
                    work_item.task,
                    work_item.path
                );
                e
            })?;
            let len = bytes.len();
            chunk_bytes.extend(bytes);
            limiter.receive_len(len).await;
        }

        Result::<(), Error>::Ok(())
    };

    select! {
        r = download_future => {
            r?;
            Ok(WorkResult{
                state: TaskResultState::Done,
                data:  chunk_bytes.into(),
                task: work_item.task,
            })
        }
        _ = work_item.cancel.cancelled() => {
            log::info!("[do work] work canceled, url: {}, range: {}, path: {:?}", url, work_item.task, work_item.path);
            Ok(WorkResult{
                state: TaskResultState::Canceled,
                data: chunk_bytes.into(),
                task: work_item.task,
            })
        }
    }
}
#[cfg(test)]
mod test {
    use crate::limiter::SpeedLimiter;

    use super::*;
    use reqwest::Url;

    #[tokio::test]
    async fn test_do_work() {
        let client = reqwest::Client::new();
        const TEST_LEN: u64 = 10;

        let url = Url::parse("https://speed.cloudflare.com/robots.txt").unwrap();
        let request = reqwest::Request::new(reqwest::Method::GET, url);

        let work_item = WorkItem {
            task: ChunkRange::new(0, TEST_LEN - 1),
            cancel: CancellationToken::new(),
            path: Arc::new(PathBuf::from("cache/test_download_data")),
            request,
        };

        let result = do_work(client, work_item, SpeedLimiter::new(None))
            .await
            .unwrap();
        assert!(matches!(result.state, TaskResultState::Done));
        assert_eq!(result.data.len(), TEST_LEN as usize);
    }
}
