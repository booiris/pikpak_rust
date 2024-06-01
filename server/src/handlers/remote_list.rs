use axum::{extract::Query, Json};
use log::error;
use pikpak_core::{api::remote_list::ApiRemoteListReq, core::file::FileStatus};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToResponse, ToSchema};

use crate::extension::auth::AuthExtractor;

use super::{get_pikpak_client, BaseResp};

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug, IntoParams)]
pub struct RemoteListReq {
    pub path: String,
}

#[derive(Serialize, Deserialize, ToSchema, ToResponse)]
pub struct RemoteListResp {
    pub files_info: Vec<RemoteListFileStatus>,
    #[serde(flatten)]
    base_resp: BaseResp,
}

#[derive(Serialize, Deserialize, ToSchema, ToResponse)]
pub struct RemoteListFileStatus {
    pub kind: String,
    pub id: String,
    pub parent_id: String,
    pub name: String,
    pub user_id: String,
    pub size: String,
    pub file_extension: String,
    pub mime_type: String,
    pub created_time: String,
    pub modified_time: String,
    pub icon_link: String,
    pub thumbnail_link: String,
    pub md5_checksum: String,
    pub hash: String,
    pub phase: String,
}

impl From<FileStatus> for RemoteListFileStatus {
    fn from(d: FileStatus) -> Self {
        Self {
            kind: d.kind,
            id: d.id,
            parent_id: d.parent_id,
            name: d.name,
            user_id: d.user_id,
            size: d.size,
            file_extension: d.file_extension,
            mime_type: d.mime_type,
            created_time: d.created_time,
            modified_time: d.modified_time,
            icon_link: d.icon_link,
            thumbnail_link: d.thumbnail_link,
            md5_checksum: d.md5_checksum,
            hash: d.hash,
            phase: d.phase,
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    params(
        RemoteListReq,
    ),
    security(
        ("jwt"=[])
    ),
    responses(
        (status = 200, description = "success, return remote list file", body = RemoteListResp),
        (status = 400, description = "request invalid", body = BaseResp)
    )
)]
pub async fn remote_list(
    AuthExtractor(token): AuthExtractor,
    Query(req): Query<RemoteListReq>,
) -> Result<Json<RemoteListResp>, BaseResp> {
    let req = ApiRemoteListReq {
        path: req.path,
        ident: token.into(),
    };
    let resp = get_pikpak_client()
        .remote_list(&req, None)
        .await
        .map_err(|e| {
            error!("[remote_list] error: {:?}", e);
            BaseResp::with_error(e)
        })?;

    Ok(Json(RemoteListResp {
        files_info: resp.files_info.into_iter().map(|x| x.into()).collect(),
        base_resp: BaseResp::default(),
    }))
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(remote_list),
    components(
        schemas(RemoteListReq, RemoteListResp, BaseResp, RemoteListFileStatus),
        responses(RemoteListResp, BaseResp)
    )
)]
pub(super) struct RemoteListApi;
