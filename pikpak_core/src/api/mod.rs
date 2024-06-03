use serde::{Deserialize, Serialize};

pub mod download;
pub mod login;
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

#[derive(Default, PartialEq, Eq, Hash, Clone, Serialize, Deserialize, Debug)]
pub struct Ident {
    pub username: String,
    pub password: String,
}
