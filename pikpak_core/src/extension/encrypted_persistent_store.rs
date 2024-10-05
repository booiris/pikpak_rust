use std::fmt::Debug;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ahash::{AHashMap, HashMap};
use parking_lot::{Mutex, RwLock};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::extension::encrypted_recycle_persistent_store::encrypt;

use super::encrypted_recycle_persistent_store::{decrypt, MemoryElem};

#[derive(Debug)]
pub struct EncryptedPersistentMemory<
    K: Hash + Eq + PartialEq + Send + 'static + Serialize + Clone + DeserializeOwned + Debug,
    V: Default + DeserializeOwned + Send + 'static + Serialize + Clone + Debug + Sync,
> {
    data: Arc<Mutex<AHashMap<K, MemoryElem<V>>>>,
    persistence_interval: Duration,
    persistence_file_path: Option<PathBuf>,
    cancel: Option<CancellationToken>,
}

impl<
        K: Hash + Eq + PartialEq + Send + 'static + Serialize + Clone + DeserializeOwned + Debug,
        V: Default + DeserializeOwned + Send + 'static + Serialize + Clone + Debug + Sync,
    > EncryptedPersistentMemory<K, V>
{
    pub fn new(
        origin_file: Option<PathBuf>,
        persistence_file_path: Option<PathBuf>,
        persistence_interval: Option<Duration>,
    ) -> Self {
        let data = origin_file
            .and_then(|x| match std::fs::read(x.clone()) {
                Ok(data) => {
                    let data = bincode::deserialize(&data)
                        .map_err(|e| {
                            error!(
                                "Failed to deserialize origin file: {} path: {}",
                                e,
                                x.display()
                            );
                        })
                        .ok()?;
                    Some(data)
                }
                Err(e) => {
                    error!(
                        "Failed to read origin file, err: {} path: {}",
                        e,
                        x.display()
                    );
                    None
                }
            })
            .unwrap_or_default();

        let mut x = Self {
            data: Arc::new(Mutex::new(data)),
            persistence_file_path,
            persistence_interval: persistence_interval.unwrap_or(Duration::from_secs(60)),
            cancel: None,
        };
        x.persistence();
        x
    }

    pub fn get_checked(&self, key: &K, decrypt_key: &str) -> Option<Arc<RwLock<V>>> {
        let mut data = self.data.lock();
        let elem = match data.get_mut(key) {
            Some(elem) => elem,
            None => return None,
        };

        match &mut *elem {
            MemoryElem::Encrypted(encrypted_data) => {
                let data = match decrypt::<V>(encrypted_data, decrypt_key.as_bytes()) {
                    Ok(d) => d,
                    Err(_) => return None,
                };
                let data = Arc::new(RwLock::new(data));
                *elem = MemoryElem::Decrypted {
                    key: decrypt_key.as_bytes().to_vec(),
                    data: data.clone(),
                };
                Some(data)
            }
            MemoryElem::Decrypted { key: _, data } => Some(data.clone()),
        }
    }

    pub fn get(&self, key: &K, decrypt_key: &str) -> Arc<RwLock<V>> {
        let mut data = self.data.lock();
        let elem = match data.get_mut(key) {
            Some(elem) => elem,
            None => {
                let d = Arc::new(RwLock::new(V::default()));
                data.insert(
                    key.clone(),
                    MemoryElem::Decrypted {
                        key: decrypt_key.as_bytes().to_vec(),
                        data: d.clone(),
                    },
                );
                return d;
            }
        };

        match &mut *elem {
            MemoryElem::Encrypted(encrypted_data) => {
                let data = decrypt::<V>(encrypted_data, decrypt_key.as_bytes()).unwrap_or_default();
                let data = Arc::new(RwLock::new(data));
                *elem = MemoryElem::Decrypted {
                    key: decrypt_key.as_bytes().to_vec(),
                    data: data.clone(),
                };
                data
            }
            MemoryElem::Decrypted { key: _, data } => data.clone(),
        }
    }

    pub fn update_decrypt_key(&self, key: &K, old_decrypt_key: &str, new_decrypt_key: &str) {
        let mut data = self.data.lock();
        let elem = match data.get_mut(key) {
            Some(elem) => elem,
            None => {
                data.insert(
                    key.clone(),
                    MemoryElem::Decrypted {
                        key: new_decrypt_key.as_bytes().to_vec(),
                        data: Default::default(),
                    },
                );
                return;
            }
        };

        match &mut *elem {
            MemoryElem::Encrypted(d) => match decrypt::<V>(d, old_decrypt_key.as_bytes()) {
                Ok(data) => {
                    let data = Arc::new(RwLock::new(data));
                    *elem = MemoryElem::Decrypted {
                        key: new_decrypt_key.as_bytes().to_vec(),
                        data: data.clone(),
                    };
                }
                Err(e) => {
                    error!("[update_decrypt_key] Failed to decrypt data: {:?}", e);
                }
            },
            MemoryElem::Decrypted { key: _, data } => {
                let data = data.clone();
                *elem = MemoryElem::Decrypted {
                    key: new_decrypt_key.as_bytes().to_vec(),
                    data,
                };
            }
        }
    }

    pub fn unlock_all(&self, decrypt_key: &str) -> AHashMap<K, Arc<RwLock<V>>> {
        let mut res = AHashMap::new();
        let keys = self.data.lock().keys().cloned().collect::<Vec<_>>();

        for k in keys {
            let d = self.get(&k, decrypt_key);
            res.insert(k.clone(), d.clone());
        }
        res
    }

    fn persistence(&mut self) {
        if self.persistence_file_path.is_none() {
            return;
        }

        let cancel = CancellationToken::new();
        self.cancel = Some(cancel.clone());

        let persistence_file = self.persistence_file_path.clone();
        let persistence_interval = self.persistence_interval;
        let data = self.data.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        break;
                    }
                    _ = tokio::time::sleep(persistence_interval) => {}
                }

                let mut cloned_data = data
                    .lock()
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<_, _>>();

                for v in cloned_data.values_mut() {
                    if let MemoryElem::Decrypted { ref key, ref data } = v {
                        let data = data.clone();
                        let data = data.read();
                        let encrypted_data = encrypt(&*data, key);
                        if encrypted_data.is_empty() {
                            continue;
                        }
                        *v = MemoryElem::Encrypted(encrypted_data);
                    }
                }

                if let Ok(data) = bincode::serialize(&cloned_data) {
                    if let Err(e) =
                        tokio::fs::write(persistence_file.as_ref().unwrap(), &data).await
                    {
                        error!("Failed to write persistence file: {}", e);
                    }
                }
            }
            info!("backup task canceled");
        });
    }
}

impl<
        K: Hash + Eq + PartialEq + Send + 'static + Serialize + Clone + DeserializeOwned + Debug,
        V: Default + DeserializeOwned + Send + 'static + Serialize + Clone + Debug + Sync,
    > Drop for EncryptedPersistentMemory<K, V>
{
    fn drop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            cancel.cancel();
        }
    }
}

#[cfg(test)]
mod test {
    use serde::Deserialize;

    use super::*;

    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    struct TestData {
        a: i32,
        b: String,
    }

    #[tokio::test]
    async fn test_encrypted_persistent_memory() {
        std::fs::create_dir_all("cache").unwrap();

        let data = EncryptedPersistentMemory::<String, TestData>::new(
            None,
            Some(PathBuf::from("cache/test_hashmap.bin")),
            Some(Duration::from_secs(3)),
        );

        {
            let d = data.get(&"test1".to_string(), "asd");
            let mut d: parking_lot::lock_api::RwLockWriteGuard<parking_lot::RawRwLock, TestData> =
                d.write();
            d.a = 1;
            d.b = "hello".to_string();
        }

        {
            let d = data.get(&"test2".to_string(), "cccc");
            let mut d = d.write();
            d.a = 2;
            d.b = "world".to_string();
        }

        {
            let d = data.get(&"test1".to_string(), "asd");
            let mut d = d.write();
            d.a = 3;
        }

        {
            let d = data.get_checked(&"test3".to_string(), "ssss");
            assert!(d.is_none());
        }

        println!("{:?}", data.data.lock());

        tokio::time::sleep(Duration::from_secs(4)).await;

        drop(data);

        tokio::time::sleep(Duration::from_secs(1)).await;

        let data = EncryptedPersistentMemory::<String, TestData>::new(
            Some(PathBuf::from("cache/test_hashmap.bin")),
            None,
            None,
        );

        println!("{:?}", data.data.lock());

        {
            let d = data.get(&"test1".to_string(), "asd");
            let d = d.read();

            println!("{:?}", d);

            assert_eq!(d.a, 3);
            assert_eq!(d.b, "hello");
        }

        {
            let d = data.get(&"test2".to_string(), "cccc");
            let d = d.read();
            assert_eq!(d.a, 2);
            assert_eq!(d.b, "world");
        }
    }
}
