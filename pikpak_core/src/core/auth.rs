use std::time::Duration;

use ahash::HashMapExt;
use anyhow::Context;
use oauth2::{reqwest::async_http_client, RefreshToken, TokenResponse};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

use crate::{
    api::Ident, error::Error, extension::expire_store::ExpireStoreMemory, utils::secret::Password,
    CLIENT_ID, CLIENT_SECRET,
};

use super::ApiClient;

lazy_static::lazy_static! {
   pub(crate) static ref AUTH_TOKEN: ExpireStoreMemory<Ident,AuthTokenType> = ExpireStoreMemory::default();
}

#[derive(Default, Debug, Clone)]
pub struct AuthTokenType {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Duration,
    pub captcha_token: String,
    pub captcha_refresh_token: String,
}

impl ApiClient<'_> {
    pub(crate) async fn access_token(&self) -> Result<String, Error> {
        let token = self.auth_token().await?;
        Ok(token.access_token)
    }

    pub(crate) async fn auth_token(&self) -> Result<AuthTokenType, Error> {
        let token = AUTH_TOKEN.get(self.ident);
        if let Some((token, _)) = token {
            return Ok((*token).clone());
        }
        trace!("auth token not found, update token again");
        let mut token = self.login().await.context("auth_token")?;

        let resp = self
            .client
            .inner
            .oauth2_client
            .exchange_refresh_token(&RefreshToken::new(token.captcha_refresh_token.clone()))
            .request_async(async_http_client)
            .await
            .map_err(|e| Error::Oauth2Error(format!("{:?}", e)))?;

        token.access_token = resp.access_token().secret().to_string();
        token.refresh_token = resp.refresh_token().map(|x| x.secret().clone());
        token.expires_in = resp.expires_in().unwrap_or_default();

        AUTH_TOKEN.set(
            self.ident.clone(),
            token.clone(),
            Some(token.expires_in / 2),
        );
        Ok(token)
    }

    async fn login(&self) -> Result<AuthTokenType, Error> {
        let captcha_token = self.get_captcha_token().await.context("[login]")?;
        let req = LoginReq {
            captcha_token: captcha_token.clone(),
            client_id: CLIENT_ID.to_string(),
            client_secret: CLIENT_SECRET.to_string(),
            username: self.ident.username.clone(),
            password: self.ident.password.clone(),
        };

        debug!("[login] login req {:#?}", req);

        let resp = self
            .http_client()
            .post("https://user.mypikpak.com/v1/auth/signin")
            .query(&[("client_id", CLIENT_ID)])
            .json(&req)
            .send()
            .await
            .context("[login] send req error")?
            .text()
            .await?;

        debug!("[login] login resp {:#?}", resp);

        let resp = serde_json::from_str::<LoginResp>(&resp).context("[login] get data error")?;

        Ok(AuthTokenType {
            access_token: resp.access_token,
            refresh_token: Some(resp.refresh_token.clone()),
            expires_in: Duration::from_secs(3600),
            captcha_token,
            captcha_refresh_token: resp.refresh_token,
        })
    }

    async fn get_captcha_token(&self) -> Result<String, Error> {
        let mut meta = ahash::HashMap::new();
        meta.insert("email".to_string(), self.ident.username.clone());

        let req = CaptchaTokenReq {
            action: "POST:signin".to_string(),
            client_id: CLIENT_ID.to_string(),
            device_id: self.client.inner.device_id.clone(),
            meta,
        };

        debug!("[get_captcha_token] req {:#?}", req);

        let resp = self
            .http_client()
            .post("https://user.mypikpak.com/v1/shield/captcha/init")
            .query(&[("client_id", CLIENT_ID)])
            .json(&req)
            .send()
            .await
            .context("[get_captcha_token] send req error")?
            .text()
            .await
            .context("[get_captcha_token] get data error")?;

        debug!("[get_captcha_token] login resp {:#?}", resp);

        let resp = serde_json::from_str::<CaptchaTokenResp>(&resp)
            .context("[get_captcha_token] parse json error")?;

        if let Some(url) = resp.url {
            return Err(Error::Oauth2Error("need captcha, url: ".to_string() + &url));
        }

        if resp.captcha_token.is_none() {
            return Err(Error::Oauth2Error("captcha token is none".to_string()));
        }

        Ok(resp.captcha_token.unwrap())
    }
}

#[derive(Debug, Clone, Serialize)]
struct CaptchaTokenReq {
    action: String,
    client_id: String,
    device_id: String,
    meta: ahash::HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CaptchaTokenResp {
    captcha_token: Option<String>,
    // expires_in: Duration,
    url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct LoginReq {
    captcha_token: String,
    client_id: String,
    client_secret: String,
    username: String,
    password: Password,
}

#[derive(Debug, Clone, Deserialize)]
struct LoginResp {
    access_token: String,
    refresh_token: String,
}

#[cfg(test)]
#[cfg(feature = "__local_test")]
mod test {
    use crate::test::{test_client, test_ident};
    use tracing::info;

    #[cfg(feature = "__local_test")]
    #[tokio::test]
    async fn test_login() {
        let client = test_client();
        let resp = client.api(&test_ident(), &None).login().await;
        info!("{:#?}", resp);
    }

    #[cfg(feature = "__local_test")]
    #[tokio::test]
    async fn test_get_captcha_token() {
        let client = test_client();
        let resp = client.api(&test_ident(), &None).get_captcha_token().await;
        info!("{:#?}", resp);
    }
}
