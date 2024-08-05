use std::sync::Arc;

use ahash::AHashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{
    core::downloader::{FileID, InnerDownloadStatus},
    extension::encrypted_persistent_store::EncryptedPersistentMemory,
};

use super::{ReadFromFile, UserName};

#[derive(Debug, Clone)]
pub struct PikPakDownloadInfo {
    data: Arc<EncryptedPersistentMemory<UserName, PikPakDownloadInfoElement>>,
    decrypt_key: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct PikPakDownloadInfoElement {
    pub download_info: Arc<RwLock<AHashMap<FileID, InnerDownloadStatus>>>,
}

impl PikPakDownloadInfo {
    pub fn get(&self, key: &UserName) -> Arc<RwLock<PikPakDownloadInfoElement>> {
        self.data.get(key, &self.decrypt_key)
    }

    pub fn get_all(&self) -> AHashMap<UserName, Arc<RwLock<PikPakDownloadInfoElement>>> {
        self.data.unlock_all(&self.decrypt_key)
    }
}

impl ReadFromFile for PikPakDownloadInfo {
    fn read_from_file(base_dir: &std::path::Path, decrypt_key: Option<String>) -> Self {
        let path = base_dir.join(Self::cache_file_name());
        let store = EncryptedPersistentMemory::new(Some(path.clone()), Some(path), None);
        Self {
            data: Arc::new(store),
            decrypt_key: decrypt_key.unwrap(),
        }
    }

    fn cache_file_name() -> &'static str {
        "pikpak_download_info_cache.bin"
    }
}
