use std::path::Path;

use anyhow::anyhow;
use path_clean::clean;

use crate::error::Error;

use super::ApiClient;

impl ApiClient<'_> {
    pub async fn download(&self, path: &str, output_dir: &Path) -> Result<(), Error> {
        let path_id = self.get_path_id_use_cache(path).await?;

        match path_id {
            super::folder::FileIDType::File(id) => {
                self.handle_single_file(path, id, output_dir).await?
            }
            super::folder::FileIDType::Folder(_) => todo!(),
        };

        Ok(())
    }

    async fn handle_single_file(
        &self,
        path: &str,
        file_id: String,
        output_dir: &Path,
    ) -> Result<(), Error> {
        let file_name = clean(path)
            .file_name()
            .ok_or(Error::RequestError(anyhow!("invalid path, path: {}", path)))?
            .to_str()
            .ok_or(Error::RequestError(anyhow!(
                "parse file name error, path: {}",
                path
            )))?;

        Ok(())
    }
}
