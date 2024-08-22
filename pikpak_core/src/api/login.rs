use crate::{
    api::Ident, core::auth::AuthTokenType, error::Error, utils::secret::Password, PkiPakApiClient,
};

#[derive(Default, Debug)]
pub struct ApiLoginReq {
    pub username: String,
    pub password: Password,
}

impl PkiPakApiClient {
    pub async fn login(&self, req: &ApiLoginReq) -> Result<AuthTokenType, Error> {
        let token = self
            .api(
                &Ident {
                    username: req.username.clone(),
                    password: req.password.clone(),
                },
                &None,
            )
            .auth_token()
            .await?;

        Ok(token)
    }
}

#[cfg(test)]
#[cfg(feature = "__local_test")]
mod test {
    use super::*;
    use dotenvy_macro::dotenv;
    use tracing::info;

    #[cfg(feature = "__local_test")]
    #[tokio::test]
    async fn test_login() {
        use crate::test::test_client;

        let req = ApiLoginReq {
            username: dotenv!("username").to_string(),
            password: dotenv!("password").to_string().into(),
        };

        let resp = test_client().login(&req).await;
        info!("{:#?}", resp);
    }
}
