use ahash::AHashMap;

use crate::{
    core::downloader::{DownloadStatus, DownloadStatusKind, FileID},
    error::Error,
    extension::api_option::ApiOption,
    PkiPakApiClient,
};

use super::Ident;

#[derive(Default, Debug)]
pub struct ApiMgetDownloadStatusReq {
    pub filter: Option<Vec<DownloadStatusKind>>,
    pub ident: Ident,
}

#[derive(Default, Debug, Clone)]
pub struct ApiMgetDownloadStatusResp {
    pub download_info: AHashMap<FileID, DownloadStatus>,
}

impl PkiPakApiClient {
    pub async fn mget_download_status(
        &self,
        req: &ApiMgetDownloadStatusReq,
        _option: Option<ApiOption>,
    ) -> Result<ApiMgetDownloadStatusResp, Error> {
        Ok(ApiMgetDownloadStatusResp {
            download_info: self
                .inner
                .downloader()
                .get_status_by_filter(&req.filter, &req.ident),
        })
    }
}
