use axum::Json;
use pikpak_core::api::download_resume::ApiDownloadResumeReq;
use serde::{Deserialize, Serialize};
use tracing::error;
use utoipa::{ToResponse, ToSchema};

use crate::extension::auth::AuthExtractor;

use super::{get_pikpak_client, BaseResp};

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct DownloadResumeReq {
    file_id: String,
}

#[derive(Serialize, Deserialize, ToSchema, ToResponse)]
pub struct DownloadResumeResp {
    #[serde(flatten)]
    base_resp: BaseResp,
}

#[utoipa::path(
    post,
    path = "",
    request_body = DownloadResumeReq,
    security(
        ("jwt"=[])
    ),
    responses(
        (status = 200, description = "success", body = DownloadResumeResp),
        (status = 400, description = "request invalid", body = BaseResp)
    )
)]
pub async fn download_resume(
    AuthExtractor(token): AuthExtractor,
    Json(req): Json<DownloadResumeReq>,
) -> Result<Json<DownloadResumeResp>, BaseResp> {
    let req = ApiDownloadResumeReq {
        file_id: req.file_id,
        ident: token.into(),
    };
    get_pikpak_client()
        .download_resume(&req, None)
        .await
        .map_err(|e| {
            error!("[download_resume] error: {:?}", e);
            BaseResp::with_error(e)
        })?;
    Ok(Json(DownloadResumeResp {
        base_resp: BaseResp::default(),
    }))
}

#[cfg(feature = "utoipa")]
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(download_resume),
    components(
        schemas(DownloadResumeReq, DownloadResumeResp, BaseResp),
        responses(DownloadResumeResp, BaseResp)
    )
)]
pub(super) struct DownloadResumeApi;
