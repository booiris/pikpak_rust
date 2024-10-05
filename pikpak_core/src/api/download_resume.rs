use crate::{error::Error, extension::api_option::ApiOption, PkiPakApiClient};

use super::Ident;

#[derive(Default, Debug)]
pub struct ApiDownloadResumeReq {
    pub file_id: String,
    pub ident: Ident,
}

impl PkiPakApiClient {
    #[tracing::instrument(skip_all)]
    pub async fn download_resume(
        &self,
        req: &ApiDownloadResumeReq,
        _option: Option<ApiOption>,
    ) -> Result<(), Error> {
        self.inner
            .downloader()
            .resume_download(&req.file_id, &req.ident)
    }
}
