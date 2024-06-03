use std::sync::Arc;

use ahash::HashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{
    core::folder::FileIDType, extension::encrypted_persistent_store::EncryptedPersistentMemory,
};

use super::ReadFromFile;

#[derive(Debug)]
pub struct PikPakFileIdCache(EncryptedPersistentMemory<String, PikPakFileIdCacheElement>);

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct PikPakFileIdCacheElement {
    pub file_id_map: HashMap<String, FileIDType>,
}

impl PikPakFileIdCache {
    pub fn get(&self, key: &String, decrypt_key: &str) -> Arc<RwLock<PikPakFileIdCacheElement>> {
        self.0.get(key, decrypt_key)
    }

    pub fn get_checked(
        &self,
        key: &String,
        decrypt_key: &str,
    ) -> Option<Arc<RwLock<PikPakFileIdCacheElement>>> {
        self.0.get_checked(key, decrypt_key)
    }
}

impl ReadFromFile for PikPakFileIdCache {
    fn read_from_file(base_dir: &std::path::Path) -> Self {
        let path = base_dir.join(Self::cache_file_name());
        let store = EncryptedPersistentMemory::new(Some(path.clone()), Some(path), None, None);
        Self(store)
    }

    fn cache_file_name() -> &'static str {
        "pikpak_file_id_cache.bin"
    }
}
