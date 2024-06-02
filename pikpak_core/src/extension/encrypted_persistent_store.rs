use std::fmt::Debug;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ahash::HashMap;
use chrono::DateTime;
use chrono::Utc;
use parking_lot::Mutex;
use ring::aead::Aad;
use ring::{
    aead::{self, BoundKey, Nonce, NonceSequence, NONCE_LEN},
    digest,
    error::Unspecified,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Serialize, Deserialize, Debug, Clone)]

pub enum MemoryElem<T> {
    Encrypted(Vec<u8>),
    Decrypted { key: Vec<u8>, data: Arc<Mutex<T>> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncryptedPersistentMemoryElem<T> {
    data: MemoryElem<T>,
    expire_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct EncryptedPersistentMemory<K, V> {
    data: Arc<Mutex<HashMap<K, EncryptedPersistentMemoryElem<V>>>>,
    refresh_time: Duration,
    persistence_interval: Duration,
    persistence_file: Option<PathBuf>,
    cancel: Option<mpsc::Sender<()>>,
}

impl<
        K: Hash + Eq + PartialEq + Send + 'static + Serialize + Clone + DeserializeOwned + Debug,
        V: Default + DeserializeOwned + Send + 'static + Serialize + Clone + Debug,
    > EncryptedPersistentMemory<K, V>
{
    pub fn new(
        origin_file: Option<PathBuf>,
        persistence_file: Option<PathBuf>,
        refresh_time: Option<Duration>,
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
            data: Arc::new(Mutex::new(data)),
            persistence_file,
            refresh_time: refresh_time.unwrap_or(Duration::from_secs(60 * 60 * 24 * 30)),
            persistence_interval: persistence_interval.unwrap_or(Duration::from_secs(60)),
            cancel: None,
        };
        x.persistence();
        x
    }

    pub fn get_checked(&self, key: &K, decrypt_key: &str) -> Option<Arc<Mutex<V>>> {
        let mut data = self.data.lock();
        let elem = match data.get_mut(key) {
            Some(elem) => elem,
            None => return None,
        };

        elem.expire_at = Utc::now() + self.refresh_time;

        match &mut elem.data {
            MemoryElem::Encrypted(encrypted_data) => {
                let data = match decrypt::<V>(encrypted_data, decrypt_key.as_bytes()) {
                    Ok(d) => d,
                    Err(_) => return None,
                };
                let data = Arc::new(Mutex::new(data));
                elem.data = MemoryElem::Decrypted {
                    key: decrypt_key.as_bytes().to_vec(),
                    data: data.clone(),
                };
                Some(data)
            }
            MemoryElem::Decrypted { key: _, data } => Some(data.clone()),
        }
    }

    pub fn get(&self, key: &K, decrypt_key: &str) -> Arc<Mutex<V>> {
        let mut data = self.data.lock();
        let elem = match data.get_mut(key) {
            Some(elem) => elem,
            None => {
                let d = Arc::new(Mutex::new(V::default()));
                data.insert(
                    key.clone(),
                    EncryptedPersistentMemoryElem {
                        data: MemoryElem::Decrypted {
                            key: decrypt_key.as_bytes().to_vec(),
                            data: d.clone(),
                        },
                        expire_at: Utc::now() + self.refresh_time,
                    },
                );
                return d;
            }
        };

        elem.expire_at = Utc::now() + self.refresh_time;

        match &mut elem.data {
            MemoryElem::Encrypted(encrypted_data) => {
                let data = decrypt::<V>(encrypted_data, decrypt_key.as_bytes()).unwrap_or_default();
                let data = Arc::new(Mutex::new(data));
                elem.data = MemoryElem::Decrypted {
                    key: decrypt_key.as_bytes().to_vec(),
                    data: data.clone(),
                };
                data
            }
            MemoryElem::Decrypted { key: _, data } => data.clone(),
        }
    }

    fn persistence(&mut self) {
        if self.persistence_file.is_none() {
            return;
        }

        let (tx, mut rx) = mpsc::channel::<()>(1);
        self.cancel = Some(tx);

        let persistence_file = self.persistence_file.clone();
        let persistence_interval = self.persistence_interval;
        let data = self.data.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = rx.recv() => {
                        break;
                    }
                    _ = tokio::time::sleep(persistence_interval) => {}
                }

                let mut cloned_data = {
                    let mut data = data.lock();
                    data.retain(|_, v| v.expire_at > Utc::now());

                    let cloned_data = data
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<HashMap<_, _>>();
                    cloned_data
                };

                for v in cloned_data.values_mut() {
                    if let MemoryElem::Decrypted { ref key, ref data } = v.data {
                        let data = data.clone();
                        let data = data.lock();
                        let encrypted_data = encrypt(&*data, key);
                        if encrypted_data.is_empty() {
                            v.expire_at = chrono::DateTime::default();
                            continue;
                        }
                        v.data = MemoryElem::Encrypted(encrypted_data);
                    }
                }
                cloned_data.retain(|_, v| v.expire_at > Utc::now());

                if let Ok(data) = bincode::serialize(&cloned_data) {
                    if let Err(e) =
                        tokio::fs::write(persistence_file.as_ref().unwrap(), &data).await
                    {
                        log::error!("Failed to write persistence file: {}", e);
                    }
                }
            }
            log::info!("Persistence task canceled");
        });
    }
}

impl<T, U> Drop for EncryptedPersistentMemory<T, U> {
    fn drop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            tokio::spawn(async move {
                let _ = cancel.send(()).await;
            });
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

fn decrypt<T: Default + DeserializeOwned>(data: &mut [u8], key: &[u8]) -> Result<T, ()> {
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

fn encrypt<T: Serialize>(data: &T, key: &[u8]) -> Vec<u8> {
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
        let data = EncryptedPersistentMemory::<String, TestData>::new(
            None,
            Some(PathBuf::from("cache/test.bin")),
            None,
            Some(Duration::from_secs(3)),
        );

        {
            let d = data.get(&"test1".to_string(), "asd");
            let mut d = d.lock();
            d.a = 1;
            d.b = "hello".to_string();
        }

        {
            let d = data.get(&"test2".to_string(), "cccc");
            let mut d = d.lock();
            d.a = 2;
            d.b = "world".to_string();
        }

        {
            let d = data.get(&"test1".to_string(), "asd");
            let mut d = d.lock();
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
            Some(PathBuf::from("cache/test.bin")),
            None,
            None,
            None,
        );

        println!("{:?}", data.data.lock());

        {
            let d = data.get(&"test1".to_string(), "asd");
            let d = d.lock();

            println!("{:?}", d);

            assert_eq!(d.a, 3);
            assert_eq!(d.b, "hello");
        }

        {
            let d = data.get(&"test2".to_string(), "cccc");
            let d = d.lock();
            assert_eq!(d.a, 2);
            assert_eq!(d.b, "world");
        }
    }
}
