use axum::Json;
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::extension::auth::AuthExtractor;

use super::BaseResp;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct DownloadBeginReq {
    path: String,
    file_id: String,
}

#[derive(Serialize, Deserialize, ToSchema, ToResponse)]
pub struct DownloadBeginResp {
    #[serde(flatten)]
    base_resp: BaseResp,
}

#[utoipa::path(
    post,
    path = "",
    request_body = DownloadBeginReq,
    security(
        ("jwt"=[])
    ),
    responses(
        (status = 200, description = "success", body = DownloadBeginResp),
        (status = 400, description = "request invalid", body = BaseResp)
    )
)]
pub async fn download_begin(
    AuthExtractor(token): AuthExtractor,
    Json(req): Json<DownloadBeginReq>,
) -> Result<Json<DownloadBeginResp>, BaseResp> {
    Ok(Json(DownloadBeginResp {
        base_resp: BaseResp::default(),
    }))
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(download_begin),
    components(
        schemas(DownloadBeginReq, DownloadBeginResp, BaseResp),
        responses(DownloadBeginResp, BaseResp)
    )
)]
pub(super) struct DownloadBeginApi;
