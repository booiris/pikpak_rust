use std::time::Duration;

use oauth2::{
    reqwest::async_http_client, ResourceOwnerPassword, ResourceOwnerUsername, TokenResponse,
};
use tracing::{error, trace};

use crate::{api::Ident, core::token::AUTH_TOKEN, error::Error, PkiPakApiClient};

#[derive(Default, Debug)]
pub struct ApiLoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Default, Debug, Clone)]
pub struct AuthTokenType {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Duration,
}

impl PkiPakApiClient {
    pub async fn login(&self, req: &ApiLoginReq) -> Result<AuthTokenType, Error> {
        let resp = self
            .inner
            .oauth2_client
            .exchange_password(
                &ResourceOwnerUsername::new(req.username.clone()),
                &ResourceOwnerPassword::new(req.password.clone()),
            )
            .request_async(async_http_client)
            .await
            .map_err(|e| {
                error!("[pikpak core login] {:?}", e);
                Error::Oauth2Error(format!("{:?}", e))
            })?;

        trace!("[pikpak core login] {:#?}", resp);

        let token = AuthTokenType {
            access_token: resp.access_token().secret().to_string(),
            refresh_token: resp.refresh_token().map(|x| x.secret().clone()),
            expires_in: resp.expires_in().unwrap_or_default(),
        };

        AUTH_TOKEN.set(
            Ident {
                username: req.username.clone(),
                password: req.password.clone(),
            },
            token.clone(),
            Some(token.expires_in / 2),
        );

        Ok(token)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use dotenv_codegen::dotenv;
    use tracing::info;

    #[cfg(feature = "__local_test")]
    #[tokio::test]
    async fn test_login() {
        use crate::test::test_client;

        let req = ApiLoginReq {
            username: dotenv!("username").to_string(),
            password: dotenv!("password").to_string(),
        };

        let resp = test_client().login(&req).await;
        info!("{:#?}", resp);
    }
}
