use axum::Json;
use pikpak_core::api::download_remove::ApiDownloadRemoveReq;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::{ToResponse, ToSchema};

use crate::extension::auth::AuthExtractor;

use super::{get_pikpak_client, BaseResp};

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct DownloadRemoveReq {
    file_id: String,
    need_remove_file: bool,
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
    info!("[download_remove] req: {:?}", req);

    let req = ApiDownloadRemoveReq {
        ident: token.into(),
        file_id: req.file_id,
        need_remove_file: req.need_remove_file,
    };

    get_pikpak_client()
        .download_remove(&req, None)
        .await
        .map_err(|e| {
            error!("[download_remove] error: {:?}", e);
            BaseResp::with_error(e)
        })?;

    Ok(Json(DownloadRemoveResp {
        base_resp: BaseResp::default(),
    }))
}

#[cfg(feature = "utoipa")]
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(download_remove),
    components(
        schemas(DownloadRemoveReq, DownloadRemoveResp, BaseResp),
        responses(DownloadRemoveResp, BaseResp)
    )
)]
pub(super) struct DownloadRemoveApi;
