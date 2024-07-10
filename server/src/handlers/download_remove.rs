use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::extension::auth::AuthExtractor;

use super::BaseResp;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct DownloadRemoveReq {
    path: String,
    file_id: String,
    is_remove_local_file: bool,
}

#[derive(Serialize, Deserialize, ToSchema, ToResponse)]
pub struct DownloadRemoveResp {
    #[serde(flatten)]
    base_resp: BaseResp,
}

#[utoipa::path(
    post,
    path = "",
    request_body = DownloadRemoveReq,
    security(
        ("jwt"=[])
    ),
    responses(
        (status = 200, description = "success", body = DownloadRemoveResp),
        (status = 400, description = "request invalid", body = BaseResp)
    )
)]
pub async fn download_remove(
    AuthExtractor(token): AuthExtractor,
    Json(req): Json<DownloadRemoveReq>,
) -> Result<Json<DownloadRemoveResp>, BaseResp> {
    Ok(Json(DownloadRemoveResp {
        base_resp: BaseResp::default(),
    }))
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(download_remove),
    components(
        schemas(DownloadRemoveReq, DownloadRemoveResp, BaseResp),
        responses(DownloadRemoveResp, BaseResp)
    )
)]
pub(super) struct DownloadRemoveApi;
