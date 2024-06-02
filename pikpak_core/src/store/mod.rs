use std::path::{Path, PathBuf};

use pikpak_file_id_cache::PikPakFileIdCache;

pub mod pikpak_file_id_cache;

pub struct Store {
    pub cache_dir: PathBuf,
    pub pikpak_file_id_cache: PikPakFileIdCache,
}

trait ReadFromFile {
    fn read_from_file(base_dir: &Path) -> Self;
    fn cache_file_name() -> &'static str;
}

impl Store {
    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        let cache_dir = cache_dir.unwrap_or_else(|| PathBuf::from("cache"));

        if !cache_dir.is_dir() {
            std::fs::create_dir_all(&cache_dir).expect("create cache dir failed");
        }

        Self {
            cache_dir: cache_dir.clone(),
            pikpak_file_id_cache: read(&cache_dir),
        }
    }
}

fn read<T: ReadFromFile>(base_dir: &Path) -> T {
    T::read_from_file(base_dir)
}
