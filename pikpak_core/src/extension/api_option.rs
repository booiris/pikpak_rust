use std::time::Duration;

use reqwest::RequestBuilder;
use tracing::warn;

use crate::{core::ApiClient, error::Error};

#[derive(Debug, Clone, Default)]
pub struct ApiOption {
    /// if None, no retry, if Some(0), retry forever
    pub retry_times: Option<usize>,
    pub retry_sleep_duration: Option<Duration>,
    pub timeout: Option<Duration>,
}

impl ApiOption {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn retry_times(mut self, retry_times: usize) -> Self {
        self.retry_times = Some(retry_times);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn retry_sleep_duration(mut self, retry_sleep_duration: Duration) -> Self {
        self.retry_sleep_duration = Some(retry_sleep_duration);
        self
    }
}

pub(crate) trait ApiSend {
    fn api_send(
        self,
        client: &ApiClient<'_>,
    ) -> impl std::future::Future<Output = Result<reqwest::Response, Error>> + Send;
}

impl ApiSend for RequestBuilder {
    async fn api_send(mut self, client: &ApiClient<'_>) -> Result<reqwest::Response, Error> {
        let option = client.option.clone().unwrap_or_default();

        if let Some(time_out) = option.timeout {
            self = self.timeout(time_out);
        }

        if let Some(retry_times) = option.retry_times {
            let mut cnt = 0;
            let sleep_time = option
                .retry_sleep_duration
                .unwrap_or(Duration::from_secs(1));
            if retry_times == 0 {
                loop {
                    // 更新 token 报错就不重试了
                    let token = client.auth_token().await?;
                    match self
                        .try_clone()
                        .ok_or(Error::CloneRequestError(format!("loop times: {}", cnt)))?
                        .bearer_auth(token)
                        .send()
                        .await
                    {
                        Ok(resp) => return Ok(resp),
                        Err(e) => {
                            tokio::time::sleep(sleep_time).await;
                            warn!("request failed, err: {}, retry times: {}", e, cnt + 1);
                            cnt += 1;
                        }
                    }
                }
            } else {
                for i in 0..retry_times {
                    // 更新 token 报错就不重试了
                    let token = client.auth_token().await?;
                    match self
                        .try_clone()
                        .ok_or(Error::CloneRequestError(format!("loop times: {}", cnt)))?
                        .bearer_auth(token)
                        .send()
                        .await
                    {
                        Ok(resp) => return Ok(resp),
                        Err(e) => {
                            if i + 1 == retry_times {
                                return Err(Error::NetworkError(e));
                            }
                            tokio::time::sleep(sleep_time).await;
                            warn!("request failed, err: {}, retry times: {}", e, i + 1);
                        }
                    }
                }
                unreachable!("retry loop should return before this line");
            }
        } else {
            let token = client.auth_token().await?;
            self = self.bearer_auth(token);
            self.send().await.map_err(Error::NetworkError)
        }
    }
}
