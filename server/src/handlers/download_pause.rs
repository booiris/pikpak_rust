use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::extension::auth::AuthExtractor;

use super::BaseResp;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct DownloadPauseReq {
    path: String,
    file_id: String,
}

#[derive(Serialize, Deserialize, ToSchema, ToResponse)]
pub struct DownloadPauseResp {
    #[serde(flatten)]
    base_resp: BaseResp,
}

#[utoipa::path(
    post,
    path = "",
    request_body = DownloadPauseReq,
    security(
        ("jwt"=[])
    ),
    responses(
        (status = 200, description = "success", body = DownloadPauseResp),
        (status = 400, description = "request invalid", body = BaseResp)
    )
)]
pub async fn download_pause(
    AuthExtractor(token): AuthExtractor,
    Json(req): Json<DownloadPauseReq>,
) -> Result<Json<DownloadPauseResp>, BaseResp> {
    Ok(Json(DownloadPauseResp {
        base_resp: BaseResp::default(),
    }))
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(download_pause),
    components(
        schemas(DownloadPauseReq, DownloadPauseResp, BaseResp),
        responses(DownloadPauseResp, BaseResp)
    )
)]
pub(super) struct DownloadPauseApi;
