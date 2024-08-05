use crate::{error::Error, extension::api_option::ApiOption, PkiPakApiClient};

use super::Ident;

#[derive(Default, Debug)]
pub struct ApiDownloadPauseReq {
    pub file_id: String,
    pub ident: Ident,
}

impl PkiPakApiClient {
    pub async fn download_pause(
        &self,
        req: &ApiDownloadPauseReq,
        _option: Option<ApiOption>,
    ) -> Result<(), Error> {
        self.inner
            .downloader()
            .pause_download(&req.file_id, &req.ident);

        Ok(())
    }
}
