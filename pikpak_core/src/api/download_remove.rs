use crate::{error::Error, extension::api_option::ApiOption, PkiPakApiClient};

use super::Ident;

#[derive(Default, Debug)]
pub struct ApiDownloadRemoveReq {
    pub file_id: String,
    pub ident: Ident,
    pub need_remove_file: bool,
}

impl PkiPakApiClient {
    #[tracing::instrument(skip_all)]
    pub async fn download_remove(
        &self,
        req: &ApiDownloadRemoveReq,
        _option: Option<ApiOption>,
    ) -> Result<(), Error> {
        self.inner
            .downloader()
            .remove_download(&req.file_id, &req.ident, req.need_remove_file)
            .await;
        Ok(())
    }
}
