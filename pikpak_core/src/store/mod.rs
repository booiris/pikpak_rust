use std::path::{Path, PathBuf};

use download_status_store::PikPakDownloadInfo;
use pikpak_file_id_cache::PikPakFileIdCache;

pub mod download_status_store;
pub mod pikpak_file_id_cache;

pub(crate) type UserName = String;
pub(crate) type RemoteFilePath = String;

pub struct Store {
    pub pikpak_file_id_cache: PikPakFileIdCache,
    pub pikpak_download_info: PikPakDownloadInfo,
}

trait ReadFromFile {
    fn read_from_file(base_dir: &Path, decrypt_key: Option<String>) -> Self;
    fn cache_file_name() -> &'static str;
}

impl Store {
    pub fn new(cache_dir: Option<PathBuf>, decrypt_key: String) -> Self {
        let cache_dir = cache_dir.unwrap_or_else(|| PathBuf::from("cache"));

        if !cache_dir.is_dir() {
            std::fs::create_dir_all(&cache_dir).expect("create cache dir failed");
        }

        Self {
            pikpak_file_id_cache: read(&cache_dir, None),
            pikpak_download_info: read(&cache_dir, Some(decrypt_key)),
        }
    }
}

fn read<T: ReadFromFile>(base_dir: &Path, decrypt_key: Option<String>) -> T {
    T::read_from_file(base_dir, decrypt_key)
}
