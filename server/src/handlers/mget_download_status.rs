use axum::Json;
use pikpak_core::{
    api::mget_download_status::ApiMgetDownloadStatusReq,
    core::downloader::{DownloadStatus, DownloadStatusKind},
};
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::{IntoParams, ToResponse, ToSchema};

use crate::extension::auth::AuthExtractor;

use super::{get_pikpak_client, BaseResp};

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub enum Filter {
    Downloading,
    Paused,
    Completed,
    Waiting,
    HasError,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub enum DownloadStatusEnum {
    Downloading,
    Paused,
    Completed,
    Waiting,
    HasError(String),
}

impl From<Filter> for DownloadStatusKind {
    fn from(f: Filter) -> Self {
        match f {
            Filter::Downloading => DownloadStatusKind::Downloading,
            Filter::Paused => DownloadStatusKind::Paused,
            Filter::Completed => DownloadStatusKind::Completed,
            Filter::Waiting => DownloadStatusKind::Waiting,
            Filter::HasError => DownloadStatusKind::HasError("".into()),
        }
    }
}

impl From<DownloadStatusKind> for DownloadStatusEnum {
    fn from(f: DownloadStatusKind) -> Self {
        match f {
            DownloadStatusKind::Downloading => DownloadStatusEnum::Downloading,
            DownloadStatusKind::Paused => DownloadStatusEnum::Paused,
            DownloadStatusKind::Completed => DownloadStatusEnum::Completed,
            DownloadStatusKind::Waiting => DownloadStatusEnum::Waiting,
            DownloadStatusKind::HasError(e) => DownloadStatusEnum::HasError(e),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct DownloadStatusType {
    pub total: u64,
    pub downloaded: u64,
    pub current_speed: f64,
    pub downloaded_time: u64,

    pub file_id: String,
    pub remote_file_name: String,
    pub download_to_local_path: String,
    pub status: DownloadStatusEnum,
}

impl From<DownloadStatus> for DownloadStatusType {
    fn from(s: DownloadStatus) -> Self {
        Self {
            total: s.total,
            downloaded: s.downloaded,
            current_speed: s.current_speed,
            downloaded_time: s.downloaded_time.as_secs(),

            file_id: s.file_id,
            remote_file_name: s.remote_file_name,
            download_to_local_path: s.download_to_local_path.display().to_string(),
            status: s.status.into(),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, IntoParams)]
pub struct MgetDownloadStatusReq {
    filter: Option<Vec<Filter>>,
    // TODO: 分页
}

#[derive(Serialize, Deserialize, ToSchema, ToResponse)]
pub struct MgetDownloadStatusResp {
    #[serde(flatten)]
    base_resp: BaseResp,
    download_status: Vec<DownloadStatusType>,
}

#[utoipa::path(
    post,
    path = "",
    request_body = MgetDownloadStatusReq,
    security(
        ("jwt"=[])
    ),
    responses(
        (status = 200, description = "success", body = MgetDownloadStatusResp),
        (status = 400, description = "request invalid", body = BaseResp)
    )
)]
pub async fn mget_download_status(
    AuthExtractor(token): AuthExtractor,
    Json(req): Json<MgetDownloadStatusReq>,
) -> Result<Json<MgetDownloadStatusResp>, BaseResp> {
    let req = ApiMgetDownloadStatusReq {
        ident: token.into(),
        filter: req.filter.map(|f| f.into_iter().map(Into::into).collect()),
    };

    let resp = get_pikpak_client()
        .mget_download_status(&req, None)
        .await
        .map_err(|e| {
            error!("[mget_download_status] error: {:?}", e);
            BaseResp::with_error(e)
        })?;

    let download_status = resp.download_info.into_values().map(Into::into).collect();

    Ok(Json(MgetDownloadStatusResp {
        base_resp: BaseResp::default(),
        download_status,
    }))
}

#[cfg(feature = "utoipa")]
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(mget_download_status),
    components(
        schemas(
            MgetDownloadStatusReq,
            MgetDownloadStatusResp,
            BaseResp,
            Filter,
            DownloadStatusType,
            DownloadStatusEnum
        ),
        responses(MgetDownloadStatusResp, BaseResp)
    )
)]
pub(super) struct MgetDownloadStatusApi;
