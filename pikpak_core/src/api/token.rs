use crate::{
    api::login::ApiLoginReq, error::Error, extension::expire_store::ExpireStoreMemory,
    PkiPakApiClient,
};

use super::{login::AuthTokenType, Ident};

lazy_static::lazy_static! {
   pub(super) static ref AUTH_TOKEN: ExpireStoreMemory<Ident,AuthTokenType> = ExpireStoreMemory::default();
}

impl PkiPakApiClient {
    pub(crate) async fn auth_token(&self, ident: &Ident) -> Result<String, Error> {
        let token = AUTH_TOKEN.get(ident);
        if let Some((token, _)) = token {
            return Ok(token.access_token.to_string());
        }
        let token = self
            .login(ApiLoginReq {
                username: ident.username.clone(),
                password: ident.password.clone(),
            })
            .await?;
        AUTH_TOKEN.set(ident.clone(), token.clone(), Some(token.expires_in / 2));
        Ok(token.access_token)
    }
}
