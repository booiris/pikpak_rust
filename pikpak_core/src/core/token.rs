use tracing::trace;

use crate::{
    api::{
        login::{ApiLoginReq, AuthTokenType},
        Ident,
    },
    error::Error,
    extension::expire_store::ExpireStoreMemory,
};

use super::ApiClient;

lazy_static::lazy_static! {
   pub(crate) static ref AUTH_TOKEN: ExpireStoreMemory<Ident,AuthTokenType> = ExpireStoreMemory::default();
}

impl ApiClient<'_> {
    pub(crate) async fn auth_token(&self) -> Result<String, Error> {
        let token = AUTH_TOKEN.get(self.ident);
        if let Some((token, _)) = token {
            return Ok(token.access_token.to_string());
        }
        trace!("auth token not found, update token again");
        let token = self
            .client
            .login(&ApiLoginReq {
                username: self.ident.username.clone(),
                password: self.ident.password.clone(),
            })
            .await?;
        AUTH_TOKEN.set(
            self.ident.clone(),
            token.clone(),
            Some(token.expires_in / 2),
        );
        Ok(token.access_token)
    }
}
