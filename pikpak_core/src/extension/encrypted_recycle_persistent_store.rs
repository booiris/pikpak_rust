use std::fmt::Debug;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ahash::HashMap;
use chrono::Utc;
use parking_lot::RwLock;
use ring::aead::Aad;
use ring::{
    aead::{self, BoundKey, Nonce, NonceSequence, NONCE_LEN},
    digest,
    error::Unspecified,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use super::auto_recycle_store::AutoRecycleStore;
use super::auto_recycle_store::IntoAutoRecycleStoreElem;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub enum MemoryElem<T> {
    Encrypted(Vec<u8>),
    Decrypted { key: Vec<u8>, data: Arc<RwLock<T>> },
}
#[derive(Debug)]
pub struct EncryptedRecyclePersistentMemory<
    K: Hash + Eq + PartialEq + Send + 'static + Serialize + Clone + DeserializeOwned + Debug,
    V: Default + DeserializeOwned + Send + 'static + Serialize + Clone + Debug + Sync,
> {
    data: AutoRecycleStore<K, MemoryElem<V>>,
    persistence_interval: Duration,
    persistence_file: Option<PathBuf>,
    cancel: Option<CancellationToken>,
}

impl<
        K: Hash + Eq + PartialEq + Send + 'static + Serialize + Clone + DeserializeOwned + Debug,
        V: Default + DeserializeOwned + Send + 'static + Serialize + Clone + Debug + Sync,
    > EncryptedRecyclePersistentMemory<K, V>
{
    pub fn new(
        origin_file: Option<PathBuf>,
        persistence_file: Option<PathBuf>,
        persistence_interval: Option<Duration>,
    ) -> Self {
        let data = origin_file
            .and_then(|x| {
                let data = std::fs::read(x.clone())
                    .map_err(|e| {
                        log::error!("Failed to read origin file: {} path: {}", e, x.display());
                    })
                    .ok()?;
                let data = bincode::deserialize(&data)
                    .map_err(|e| {
                        log::error!(
                            "Failed to deserialize origin file: {} path: {}",
                            e,
                            x.display()
                        );
                    })
                    .ok()?;
                Some(data)
            })
            .unwrap_or_default();

        let mut x = Self {
            data: AutoRecycleStore::new(data),
            persistence_file,
            persistence_interval: persistence_interval.unwrap_or(Duration::from_secs(60)),
            cancel: None,
        };
        x.persistence();
        x
    }

    pub fn get_checked(&self, key: &K, decrypt_key: &str) -> Option<Arc<RwLock<V>>> {
        let mut elem = match self.data.get_mut(key) {
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
        let mut elem = match self.data.get_mut(key) {
            Some(elem) => elem,
            None => {
                let d = Arc::new(RwLock::new(V::default()));
                self.data.insert(
                    key.clone(),
                    MemoryElem::Decrypted {
                        key: decrypt_key.as_bytes().to_vec(),
                        data: d.clone(),
                    }
                    .into_recycle_elem(None),
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
        let mut elem = match self.data.get_mut(key) {
            Some(elem) => elem,
            None => {
                self.data.insert(
                    key.clone(),
                    MemoryElem::Decrypted {
                        key: new_decrypt_key.as_bytes().to_vec(),
                        data: Default::default(),
                    }
                    .into_recycle_elem(None),
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
                    log::error!("[update_decrypt_key] Failed to decrypt data: {:?}", e);
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

    fn persistence(&mut self) {
        if self.persistence_file.is_none() {
            return;
        }

        let cancel = CancellationToken::new();
        self.cancel = Some(cancel.clone());

        let persistence_file = self.persistence_file.clone();
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
                    if let MemoryElem::Decrypted { ref key, ref data } = v.data {
                        let data = data.clone();
                        let data = data.read();
                        let encrypted_data = encrypt(&*data, key);
                        if encrypted_data.is_empty() {
                            v.recycle_at = chrono::DateTime::default();
                            continue;
                        }
                        v.data = MemoryElem::Encrypted(encrypted_data);
                    }
                }
                cloned_data.retain(|_, v| v.recycle_at > Utc::now());

                if let Ok(data) = bincode::serialize(&cloned_data) {
                    if let Err(e) =
                        tokio::fs::write(persistence_file.as_ref().unwrap(), &data).await
                    {
                        log::error!("Failed to write persistence file: {}", e);
                    }
                }
            }
            log::info!("backup task canceled");
        });
    }
}

impl<
        K: Hash + Eq + PartialEq + Send + 'static + Serialize + Clone + DeserializeOwned + Debug,
        V: Default + DeserializeOwned + Send + 'static + Serialize + Clone + Debug + Sync,
    > Drop for EncryptedRecyclePersistentMemory<K, V>
{
    fn drop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            cancel.cancel();
        }
    }
}

#[derive(Default)]
pub(crate) struct NonceSeq([u8; NONCE_LEN]);

impl NonceSequence for &mut NonceSeq {
    fn advance(&mut self) -> Result<Nonce, Unspecified> {
        Nonce::try_assume_unique_for_key(&self.0)
    }
}

pub(crate) fn decrypt<T: Default + DeserializeOwned>(data: &mut [u8], key: &[u8]) -> Result<T, ()> {
    let key = digest::digest(&digest::SHA256, key);
    let unbound_key = match aead::UnboundKey::new(&aead::AES_256_GCM, key.as_ref()) {
        Ok(k) => k,
        Err(e) => {
            log::error!("[decrypt] Failed to create unbound key: {}", e);
            return Err(());
        }
    };

    let nonce = &mut NonceSeq::default();
    let mut open_key = aead::OpeningKey::new(unbound_key, nonce);
    let data = match open_key.open_in_place(Aad::empty(), data) {
        Ok(d) => d.to_vec(),
        Err(e) => {
            log::error!("[decrypt] Failed to decrypt data: {}", e);
            return Err(());
        }
    };

    match bincode::deserialize::<T>(&data) {
        Ok(d) => Ok(d),
        Err(_) => Err(()),
    }
}

pub(crate) fn encrypt<T: Serialize>(data: &T, key: &[u8]) -> Vec<u8> {
    let key = digest::digest(&digest::SHA256, key);
    let unbound_key = match aead::UnboundKey::new(&aead::AES_256_GCM, key.as_ref()) {
        Ok(k) => k,
        Err(e) => {
            log::error!("[encrypt] Failed to create unbound key: {}", e);
            return Vec::new();
        }
    };

    let nonce = &mut NonceSeq::default();
    let mut sealing_key = aead::SealingKey::new(unbound_key, nonce);
    let mut data = match bincode::serialize(data) {
        Ok(d) => d,
        Err(e) => {
            log::error!("[encrypt] Failed to serialize data: {}", e);
            return Vec::new();
        }
    };

    match sealing_key.seal_in_place_append_tag(Aad::empty(), &mut data) {
        Ok(_) => data,
        Err(e) => {
            log::error!("Failed to encrypt data: {}", e);
            Vec::new()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Default, Clone)]
    struct TestData {
        a: i32,
        b: String,
    }

    #[tokio::test]
    async fn test_encrypted_persistent_memory() {
        let data = EncryptedRecyclePersistentMemory::<String, TestData>::new(
            None,
            Some(PathBuf::from("cache/test.bin")),
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

        let data = EncryptedRecyclePersistentMemory::<String, TestData>::new(
            Some(PathBuf::from("cache/test.bin")),
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
