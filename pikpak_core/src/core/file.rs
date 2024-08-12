use ahash::AHashMap;
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::error::Error;

use super::ApiClient;

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct FileStatus {
    pub kind: String,
    pub id: String,
    pub parent_id: String,
    pub name: String,
    pub user_id: String,
    pub size: String,
    pub file_extension: String,
    pub mime_type: String,
    pub created_time: String,
    pub modified_time: String,
    pub icon_link: String,
    pub thumbnail_link: String,
    pub md5_checksum: String,
    pub hash: String,
    pub phase: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct FileType {
    pub kind: String,
    pub id: String,
    pub parent_id: String,
    pub name: String,
    pub user_id: String,
    pub size: String,
    pub revision: String,
    pub file_extension: String,
    pub mime_type: String,
    pub starred: bool,
    pub web_content_link: String,
    pub created_time: String,
    pub modified_time: String,
    pub icon_link: String,
    pub thumbnail_link: String,
    pub md5_checksum: String,
    pub hash: String,
    pub links: Links,
    pub phase: String,
    pub trashed: bool,
    pub delete_time: String,
    pub original_url: String,
    pub original_file_index: i64,
    pub space: String,
    pub writable: bool,
    pub folder_type: String,
    pub sort_name: String,
    pub user_modified_time: String,
    pub file_category: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Links {
    #[serde(rename = "application/octet-stream")]
    pub application_octet_stream: ApplicationOctetStream,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct ApplicationOctetStream {
    pub url: String,
    pub token: String,
    pub expire: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct StatusResp {
    pub next_page_token: String,
    pub files: Vec<FileStatus>,
}

impl ApiClient<'_> {
    pub async fn get_file_status_list_by_folder_id(
        &self,
        folder_id: &str,
    ) -> Result<Vec<FileStatus>, Error> {
        let mut query = AHashMap::from([
            ("thumbnail_size", "SIZE_MEDIUM".into()),
            // seems that api does not support offset and limit
            ("limit", "500".into()),
            ("parent_id", folder_id.to_owned()),
            ("with_audit", "false".into()),
            ("filters", r#"{"trashed":{"eq":false}}"#.into()),
        ]);

        let mut file_list = vec![];

        loop {
            let req = self
                .http_client()
                .get("https://api-drive.mypikpak.com/drive/v1/files")
                .query(&query);

            let resp = self.send_raw_req::<StatusResp>(req).await?;

            file_list.extend(resp.files);
            if resp.next_page_token.is_empty() {
                break;
            }
            query.insert("page_token", resp.next_page_token);
        }

        Ok(file_list)
    }

    pub async fn get_file_by_id(&self, file_id: &str) -> Result<FileType, Error> {
        let req = self.http_client().get(format!(
            "https://api-drive.mypikpak.com/drive/v1/files/{}",
            file_id
        ));

        trace!("req: {:?}", req);

        self.send_raw_req::<FileType>(req).await
    }
}

#[cfg(test)]
mod test {
    use dotenv_codegen::dotenv;
    use tracing::debug;

    use crate::test::{test_client, test_ident};

    use super::*;

    #[cfg(feature = "__local_test")]
    #[tokio::test]
    async fn test_get_file_status_list_by_folder_id() -> Result<(), Error> {
        let client = test_client();
        let resp = client
            .api(&test_ident(), &None)
            .get_file_status_list_by_folder_id("")
            .await
            .expect("get file status list by folder id error");
        debug!("{:#?}", resp);
        Ok(())
    }

    #[cfg(feature = "__local_test")]
    #[tokio::test]
    async fn test_get_file_by_id() -> Result<(), Error> {
        let client = test_client();
        let resp = client
            .api(&test_ident(), &None)
            .get_file_by_id(dotenv!("test_file_id"))
            .await
            .expect("get file by id error");
        debug!("{:#?}", resp);
        Ok(())
    }
}
