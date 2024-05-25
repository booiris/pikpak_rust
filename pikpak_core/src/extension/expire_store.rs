use std::hash::Hash;
use std::{sync::Arc, time::Duration};

use ahash::AHashMap;
use chrono::Utc;
use parking_lot::RwLock;

#[derive(Default)]
pub struct ExpireStoreMemory<K, V> {
    data: RwLock<AHashMap<K, MemoryElem<V>>>,
}

struct MemoryElem<V> {
    data: Arc<V>,
    expire_at: i64,
}

impl<K: Hash + Eq + PartialEq, V> ExpireStoreMemory<K, V> {
    pub fn get(&self, key: &K) -> Option<(Arc<V>, Option<Duration>)> {
        let data = self.data.read();
        let elem = match data.get(key) {
            Some(elem) => elem,
            None => return None,
        };
        if elem.expire_at < 0 {
            return Some((elem.data.clone(), None));
        }
        let ttl = elem
            .expire_at
            .checked_sub(Utc::now().timestamp())
            .unwrap_or(0);

        if ttl <= 0 {
            drop(data);
            let mut data = self.data.write();
            data.remove(key);
            return None;
        }

        Some((elem.data.clone(), Some(Duration::from_secs(ttl as u64))))
    }

    pub fn set(&self, key: K, val: V, ttl: Option<Duration>) {
        let mut data = self.data.write();
        let mut elem = MemoryElem {
            data: Arc::new(val),
            expire_at: -1,
        };
        if let Some(ttl) = ttl {
            elem.expire_at = Utc::now()
                .timestamp()
                .checked_add(ttl.as_secs() as i64)
                .unwrap_or_default();
        }
        data.insert(key, elem);
    }
}
