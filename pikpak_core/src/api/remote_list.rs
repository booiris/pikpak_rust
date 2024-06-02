use serde::{Deserialize, Serialize};

use crate::{
    core::file::FileStatus, error::Error, extension::api_option::ApiOption, PkiPakApiClient,
};

use super::Ident;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ApiRemoteListReq {
    pub path: String,
    pub ident: Ident,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
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

        let path_id_cache = self
            .inner
            .store
            .pikpak_file_id_cache
            .get_checked(&req.ident.username, &req.ident.password);
        let folder_id = path_id_cache.and_then(|x| x.lock().file_id_map.get(&req.path).cloned());

        let folder_id = if let Some(path_id) = folder_id {
            path_id
        } else {
            let folder_id = api.get_path_id(&req.path).await?;
            self.inner
                .store
                .pikpak_file_id_cache
                .get(&req.ident.username, &req.ident.password)
                .lock()
                .file_id_map
                .insert(req.path.clone(), folder_id.clone());
            folder_id
        };

        let infos = api
            .get_file_status_list_by_folder_id(folder_id.get_id())
            .await?;
        Ok(ApiRemoteListResp { files_info: infos })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use log::debug;

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
