use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use crate::store::UserName;

pub mod download;
pub mod download_pause;
pub mod download_remove;
pub mod download_resume;
pub mod login;
pub mod mget_download_status;
pub mod remote_list;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ErrorResp {
    pub error: String,
    pub error_code: i64,
    pub error_description: String,
    pub details: Vec<Detail>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Detail {
    #[serde(rename = "@type")]
    pub type_field: String,
    pub reason: Option<String>,
    pub locale: Option<String>,
    pub message: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum RespWrapper<T> {
    Success(T),
    Err(ErrorResp),
}

#[derive(Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct Ident {
    pub username: UserName,
    pub password: String,
}

impl Debug for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ident")
            .field("username", &self.username)
            .field("password", &"********")
            .finish()
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:********", self.username)
    }
}
