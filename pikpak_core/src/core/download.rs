// use std::path::Path;

// use anyhow::Context;
// use downloader::downloader::DownloadConfig;
// use log::{debug, error};

// use crate::{error::Error, utils::file::create_dir_if_not_exists};

// use super::ApiClient;

// impl ApiClient<'_> {
//     pub async fn download(&self, path: &String, output_dir: &Path) -> Result<(), Error> {
//         let path_id = self.get_path_id_use_cache(path).await?;

//         debug!("path_id: {:?}", path_id);

//         match path_id {
//             super::folder::FileIDType::File(id) => {
//                 self.handle_single_file(path, id, &output_dir.join(path))
//                     .await?
//             }
//             super::folder::FileIDType::Folder(_) => todo!(),
//         };

//         Ok(())
//     }

//     #[allow(clippy::await_holding_lock)]
//     async fn handle_single_file(
//         &self,
//         path: &str,
//         file_id: String,
//         output_path: &Path,
//     ) -> Result<(), Error> {
//         let file_dir = output_path
//             .parent()
//             .ok_or(anyhow::anyhow!("[download_file] failed to get parent dir"))?;
//         create_dir_if_not_exists(file_dir)?;

//         let file_info = self.get_file_by_id(&file_id).await?;

//         let mut managers = self.client.inner.download_mangers.lock();

//         if managers.get(&file_id).is_some() {
//             return Ok(());
//         }

//         let manager = self.client.inner.downloader.create_runner(DownloadConfig {
//             url: file_info.links.application_octet_stream.url,
//             download_path: output_path.into(),
//             chunk_size: None,
//             worker_num: Some(4),
//             speed_limit_per_sec: None,
//             total_size: file_info.size.parse().context("file size is invalid")?,
//         });

//         let (runner, cancel) = manager.run().await?;

//         let download_path = path.to_string();
//         let runner = tokio::spawn(async move {
//             if let Err(e) = runner.await {
//                 error!(
//                     "[download] download failed, path: {}, error: {:?}",
//                     download_path, e
//                 );
//             }
//         });

//         managers.insert(file_id.to_string(), (manager, cancel, runner));

//         Ok(())
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[cfg(feature = "__local_test")]
//     #[tokio::test]
//     async fn test_download() -> anyhow::Result<()> {
//         use anyhow::Context;
//         use dotenv_codegen::dotenv;

//         let client = crate::test::test_client();
//         client
//             .api(&crate::test::test_ident(), &None)
//             .download(
//                 &"./PikPak Tutorial.mp4".to_string(),
//                 Path::new("cache/test"),
//             )
//             .await
//             .context("download error")?;

//         let runner = client
//             .inner
//             .download_mangers
//             .lock()
//             .remove(dotenv!("download_test_file_id"))
//             .context("manager is empty")?;
//         runner.2.await.context("download failed")?;

//         Ok(())
//     }
// }
