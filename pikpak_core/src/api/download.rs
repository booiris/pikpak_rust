use std::path::PathBuf;

use crate::{
    error::Error, extension::api_option::ApiOption, utils::file::create_dir_if_not_exists,
    PkiPakApiClient,
};

use super::Ident;

#[derive(Default, Debug)]
pub struct ApiDownloadReq {
    pub path: String,
    pub output_dir: PathBuf,
    pub ident: Ident,
}

impl PkiPakApiClient {
    pub async fn download(
        &self,
        req: &ApiDownloadReq,
        option: Option<ApiOption>,
    ) -> Result<(), Error> {
        create_dir_if_not_exists(&req.output_dir)?;

        let api = self.api(&req.ident, &option);

        api.download(&req.path, &req.output_dir).await
    }
}
