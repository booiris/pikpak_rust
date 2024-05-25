use axum::Json;
use log::info;
use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::extension::auth::AuthExtractor;

use super::BaseResp;

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct RemoteListReq {}

#[derive(Serialize, Deserialize, ToSchema, ToResponse)]
pub struct RemoteListResp {
    #[serde(flatten)]
    base_resp: BaseResp,
}

#[utoipa::path(
    get,
    path = "",
    responses(
        (status = 200, description = "success, return remote list file", body = LoginResp),
        (status = 400, description = "request invalid", body = BaseResp)
    )
)]
pub async fn remote_list(
    AuthExtractor(token): AuthExtractor,
) -> Result<Json<RemoteListResp>, BaseResp> {
    info!("{:#?}", token);

    Ok(Json(RemoteListResp {
        base_resp: BaseResp::default(),
    }))
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(remote_list),
    components(
        schemas(RemoteListReq, RemoteListResp, BaseResp),
        responses(RemoteListResp, BaseResp)
    )
)]
pub(super) struct RemoteListApi;
