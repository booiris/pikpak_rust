use std::hash::Hash;
use std::{sync::Arc, time::Duration};

use ahash::HashMap;
use chrono::{DateTime, Utc};
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use tracing::info;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoRecycleStore<K: Hash + Eq + PartialEq, T> {
    data: Arc<Mutex<HashMap<K, AutoRecycleStoreElem<T>>>>,
    #[serde(skip)]
    cancel: Option<CancellationToken>,
}

const DEFAULT_REFRESH_TIME: Duration = Duration::from_secs(60 * 60 * 24 * 30);

impl<K: Hash + Eq + PartialEq + Send + 'static, T: Send + 'static> Default
    for AutoRecycleStore<K, T>
{
    fn default() -> Self {
        let mut x = Self {
            data: Arc::new(Mutex::new(HashMap::default())),
            cancel: None,
        };
        x.recycle();
        x
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutoRecycleStoreElem<T> {
    pub data: T,
    pub recycle_at: DateTime<Utc>,
    pub refresh_time: Duration,
}

pub trait IntoAutoRecycleStoreElem {
    fn into_recycle_elem(self, refresh_time: Option<Duration>) -> AutoRecycleStoreElem<Self>
    where
        Self: std::marker::Sized;
}

impl<T> IntoAutoRecycleStoreElem for T {
    fn into_recycle_elem(self, refresh_time: Option<Duration>) -> AutoRecycleStoreElem<Self>
    where
        Self: std::marker::Sized,
    {
        let refresh_time = refresh_time.unwrap_or(DEFAULT_REFRESH_TIME);
        AutoRecycleStoreElem {
            data: self,
            recycle_at: Utc::now() + refresh_time,
            refresh_time,
        }
    }
}

impl<K: Hash + Eq + PartialEq + Send + 'static, T: Send + 'static> AutoRecycleStore<K, T> {
    pub fn new(data: HashMap<K, AutoRecycleStoreElem<T>>) -> Self {
        let mut x = Self {
            data: Arc::new(Mutex::new(data)),
            cancel: None,
        };
        x.recycle();
        x
    }

    pub fn lock(&self) -> MutexGuard<'_, HashMap<K, AutoRecycleStoreElem<T>>> {
        self.data.lock()
    }

    pub fn get_mut(&self, key: &K) -> Option<MappedMutexGuard<T>> {
        let data = self.data.lock();
        if data.contains_key(key) {
            Some(MutexGuard::map(data, |x| {
                let elem = x.get_mut(key).unwrap();
                elem.recycle_at = Utc::now() + elem.refresh_time;
                &mut elem.data
            }))
        } else {
            None
        }
    }

    pub fn insert(&self, key: K, elem: AutoRecycleStoreElem<T>) {
        self.data.lock().insert(key, elem);
    }

    fn recycle(&mut self) {
        let cancel = CancellationToken::new();
        self.cancel = Some(cancel.clone());

        let data = self.data.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(360)) => {}
                    _ = cancel.cancelled() => break,
                }
                let mut data = data.lock();
                data.retain(|_, v| v.recycle_at > Utc::now());
            }
            info!("AutoRecycleStore recycle task exit");
        });
    }
}

impl<K: Hash + Eq + PartialEq, T> Drop for AutoRecycleStore<K, T> {
    fn drop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            cancel.cancel();
        }
    }
}
