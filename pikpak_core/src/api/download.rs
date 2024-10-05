use std::path::PathBuf;

use crate::{
    error::Error, extension::api_option::ApiOption, utils::file::create_dir_if_not_exists,
    PkiPakApiClient,
};

use super::Ident;

#[derive(Default, Debug)]
pub struct ApiDownloadReq {
    pub file_id: String,
    pub output_dir: PathBuf,
    pub rename: String,
    pub ident: Ident,
}

impl PkiPakApiClient {
    #[tracing::instrument(skip_all)]
    pub async fn download(
        &self,
        req: &ApiDownloadReq,
        option: Option<ApiOption>,
    ) -> Result<(), Error> {
        create_dir_if_not_exists(&req.output_dir)?;

        let download_to_local_path = req.output_dir.join(req.rename.clone());

        let api = self.api(&req.ident, &option);
        let info = api.get_file_by_id(&req.file_id).await?;
        self.inner.downloader().start_download(
            req.file_id.clone(),
            download_to_local_path,
            &req.ident,
            info.name,
            info.size.parse().map_err(|e| {
                Error::RequestError(anyhow::anyhow!(
                    "parse size error: {}, size:, {}",
                    e,
                    info.size
                ))
            })?,
        )?;
        Ok(())
    }
}
