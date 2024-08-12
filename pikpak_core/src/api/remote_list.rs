use crate::{
    core::file::FileStatus, error::Error, extension::api_option::ApiOption, PkiPakApiClient,
};

use super::Ident;

#[derive(Default, Debug)]
pub struct ApiRemoteListReq {
    pub path: String,
    pub ident: Ident,
}

#[derive(Default, Debug, Clone)]
pub struct ApiRemoteListResp {
    pub files_info: Vec<FileStatus>,
}

impl PkiPakApiClient {
    pub async fn remote_list(
        &self,
        req: &ApiRemoteListReq,
        option: Option<ApiOption>,
    ) -> Result<ApiRemoteListResp, Error> {
        let api = self.api(&req.ident, &option);

        let folder_id = api.get_path_id_use_cache(&req.path).await?;

        let infos = api
            .get_file_status_list_by_folder_id(folder_id.get_id())
            .await?;
        Ok(ApiRemoteListResp { files_info: infos })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tracing::debug;

    use crate::test::{test_client, test_ident};

    #[tokio::test]
    async fn test_remote_list() {
        let client = test_client();
        let ident = test_ident();
        let req = ApiRemoteListReq {
            path: "/".to_string(),
            ident,
        };
        let resp = client.remote_list(&req, None).await.unwrap();
        debug!("resp: {:#?}", resp);
    }
}
