use axum::Json;
use log::{error, info};
use pikpak_core::api::download::ApiDownloadReq;
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::{extension::auth::AuthExtractor, handlers::get_pikpak_client};

use super::BaseResp;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct DownloadBeginReq {
    file_id: String,
    output_dir: String,
    rename: String,
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
    info!("download begin: {:?}", req);

    let req = ApiDownloadReq {
        ident: token.into(),
        file_id: req.file_id,
        output_dir: req.output_dir.into(),
        rename: req.rename,
    };
    get_pikpak_client()
        .download(&req, None)
        .await
        .map_err(|e| {
            error!("[download_begin] error: {:?}", e);
            BaseResp::with_error(e)
        })?;

    Ok(Json(DownloadBeginResp {
        base_resp: BaseResp::default(),
    }))
}

#[cfg(feature = "utoipa")]
#[derive(utoipa::OpenApi)]
#[openapi(
    paths(download_begin),
    components(
        schemas(DownloadBeginReq, DownloadBeginResp, BaseResp),
        responses(DownloadBeginResp, BaseResp)
    )
)]
pub(super) struct DownloadBeginApi;
