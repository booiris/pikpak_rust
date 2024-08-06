use axum::Json;
use log::error;
use pikpak_core::api::download_pause::ApiDownloadPauseReq;
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::extension::auth::AuthExtractor;

use super::{get_pikpak_client, BaseResp};

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct DownloadPauseReq {
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
    let req = ApiDownloadPauseReq {
        ident: token.into(),
        file_id: req.file_id,
    };

    get_pikpak_client()
        .download_pause(&req, None)
        .await
        .map_err(|e| {
            error!("[download_pause] error: {:?}", e);
            BaseResp::with_error(e)
        })?;

    Ok(Json(DownloadPauseResp {
        base_resp: BaseResp::default(),
    }))
}

#[cfg(feature = "utoipa")]
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(download_pause),
    components(
        schemas(DownloadPauseReq, DownloadPauseResp, BaseResp),
        responses(DownloadPauseResp, BaseResp)
    )
)]
pub(super) struct DownloadPauseApi;
