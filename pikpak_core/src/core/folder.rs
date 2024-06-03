use anyhow::Context;
use log::{debug, trace};
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::Error, extension::auto_recycle_store::IntoAutoRecycleStoreElem, utils::path::slash,
};

use super::ApiClient;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct GetFolderResp {
    pub files: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FileIDType {
    #[serde(rename = "drive#file")]
    File(String),
    #[serde(rename = "drive#folder")]
    Folder(String),
}

impl FileIDType {
    pub fn get_id(&self) -> &String {
        match self {
            FileIDType::File(id) => id,
            FileIDType::Folder(id) => id,
        }
    }
}

impl ApiClient<'_> {
    pub(crate) async fn get_path_id_use_cache(&self, path: &String) -> Result<FileIDType, Error> {
        let path_id_cache = self
            .client
            .inner
            .store
            .pikpak_file_id_cache
            .get_checked(&self.ident.username, &self.ident.password);
        let folder_id = path_id_cache.and_then(|x| {
            x.read()
                .file_id_map
                .refresh(path)
                .get(path)
                .map(|x| x.data.clone())
        });

        let folder_id = if let Some(path_id) = folder_id {
            path_id
        } else {
            let folder_id = self.get_path_id(path).await?;
            self.client
                .inner
                .store
                .pikpak_file_id_cache
                .get(&self.ident.username, &self.ident.password)
                .write()
                .file_id_map
                .refresh(path)
                .insert(path.to_string(), folder_id.clone().into_elem(None));
            folder_id
        };

        Ok(folder_id)
    }

    pub(crate) async fn get_path_id(&self, path: &str) -> Result<FileIDType, Error> {
        self.get_deep_folder_id(FileIDType::Folder("".into()), path)
            .await
    }

    // 获取文件夹 id
    // dir 可以包括 /.
    // 若以 / 开头，函数会去除 /， 且会从 parent 目录开始查找
    pub(crate) async fn get_deep_folder_id(
        &self,
        mut parent_id: FileIDType,
        path: &str,
    ) -> Result<FileIDType, Error> {
        let dir_path = slash(path).context("[get_deep_folder_id]")?;
        if dir_path.is_empty() {
            return Ok(parent_id);
        }

        for dir in dir_path.split('/') {
            parent_id = self
                .get_sub_folder_id(parent_id.get_id(), dir)
                .await
                .context("[get_deep_folder_id]")?;
            debug!("get folder: {}, folder id: {:?}", dir, parent_id);
        }

        Ok(parent_id)
    }

    async fn get_sub_folder_id(&self, parent_id: &str, path: &str) -> Result<FileIDType, Error> {
        let dir = slash(path).context("get_sub_folder_id")?;

        let infos = self.get_info_by_folder_id(parent_id).await?;
        for file in infos {
            let kind = file
                .get("kind")
                .and_then(|x| x.as_str())
                .unwrap_or_default();
            let name = file
                .get("name")
                .and_then(|x| x.as_str())
                .unwrap_or_default();
            let trashed = file
                .get("trashed")
                .and_then(|x| x.as_bool())
                .unwrap_or_default();
            if name == dir && !trashed {
                let id = file
                    .get("id")
                    .ok_or(anyhow::anyhow!("[get_sub_folder_id] id not found"))?
                    .as_str()
                    .ok_or(anyhow::anyhow!(
                        "[get_sub_folder_id] id can not parse to string"
                    ))?
                    .to_string();
                if kind == "drive#folder" {
                    return Ok(FileIDType::Folder(id));
                } else {
                    return Ok(FileIDType::File(id));
                };
            }
        }

        Err(Error::RequestError(anyhow::anyhow!(
            "[get_folder_id] folder not found"
        )))
    }

    async fn get_info_by_folder_id(&self, folder_id: &str) -> Result<Vec<Value>, Error> {
        let query = [
            ("parent_id", folder_id),
            ("page_token", ""),
            ("with_audit", "false"),
            ("thumbnail_size", "SIZE_LARGE"),
            ("limit", "200"),
        ];

        let mut headers = HeaderMap::new();
        headers.insert("Country", "CN".parse().expect("parse header error"));
        headers.insert(
            "X-Peer-Id",
            self.client
                .inner
                .device_id
                .parse()
                .expect("parse header error"),
        );
        headers.insert("X-User-Region", "1".parse().expect("parse header error"));
        headers.insert("X-Alt-Capability", "3".parse().expect("parse header error"));
        headers.insert(
            "X-Client-Version-Code",
            "10083".parse().expect("parse header error"),
        );

        let req = self
            .http_client()
            .get("https://api-drive.mypikpak.com/drive/v1/files")
            .query(&query)
            .headers(headers);

        trace!("req: {:?}", req);

        self.send_raw_req::<GetFolderResp>(req)
            .await
            .map(|x| x.files)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::{test_client, test_ident};
    use dotenv_codegen::dotenv;
    use log::debug;

    #[cfg(feature = "__local_test")]
    #[tokio::test]
    async fn test_get_path_id() -> Result<(), Error> {
        let client = test_client();
        let resp = client
            .api(&test_ident(), &None)
            .get_path_id("My Pack")
            .await
            .expect("get path id error");

        debug!("{:#?}", resp);

        Ok(())
    }

    #[cfg(feature = "__local_test")]
    #[tokio::test]
    async fn test_get_folder_info_by_folder_id() -> Result<(), Error> {
        let client = test_client();
        let resp = client
            .api(&test_ident(), &None)
            .get_info_by_folder_id(dotenv!("test_folder_id"))
            .await
            .expect("get folder info by folder id error");

        debug!("{:#?}", resp);

        Ok(())
    }
}
