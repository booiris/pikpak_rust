use axum::{extract::Query, Json};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToResponse, ToSchema};

use crate::extension::auth::AuthExtractor;

use super::BaseResp;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub enum Filter {
    Downloading,
    Paused,
    Completed,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, IntoParams)]
pub struct MgetDownloadStatusReq {
    filter: Option<Filter>,
    // TODO: 分页
}

#[derive(Serialize, Deserialize, ToSchema, ToResponse)]
pub struct MgetDownloadStatusResp {
    #[serde(flatten)]
    base_resp: BaseResp,
}

#[utoipa::path(
    get,
    path = "",
    params (
        MgetDownloadStatusReq
    ),
    security(
        ("jwt"=[])
    ),
    responses(
        (status = 200, description = "success", body = DownloadPauseResp),
        (status = 400, description = "request invalid", body = BaseResp)
    )
)]
pub async fn mget_download_status(
    AuthExtractor(token): AuthExtractor,
    Query(req): Query<MgetDownloadStatusReq>,
) -> Result<Json<MgetDownloadStatusResp>, BaseResp> {
    Ok(Json(MgetDownloadStatusResp {
        base_resp: BaseResp::default(),
    }))
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(mget_download_status),
    components(
        schemas(MgetDownloadStatusReq, MgetDownloadStatusResp, BaseResp, Filter),
        responses(MgetDownloadStatusResp, BaseResp)
    )
)]
pub(super) struct MgetDownloadStatusApi;
