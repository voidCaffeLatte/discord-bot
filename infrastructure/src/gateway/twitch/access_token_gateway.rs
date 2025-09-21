use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use domain::value_object::twitch::access_token::AccessToken;

pub struct AccessTokenGateway {
    http_client: Arc<reqwest::Client>,
    app_client_id: String,
    app_client_secret: String,
    access_token: Mutex<Option<AccessToken>>,
}

impl AccessTokenGateway {
    const TOKEN_API_BASE_URL: &'static str = "https://id.twitch.tv/oauth2/token";
    const VALIDATE_API_BASE_URL: &'static str = "https://id.twitch.tv/oauth2/validate";

    pub fn new(
        http_client: Arc<reqwest::Client>,
        app_client_id: String,
        app_client_secret: String,
    ) -> Self {
        Self {
            http_client,
            app_client_id,
            app_client_secret,
            access_token: Mutex::new(None),
        }
    }

    pub async fn get_access_token(&self, at: &DateTime<Utc>) -> Result<AccessToken, Error> {
        let mut access_token_guard = self.access_token.lock().await;
        if let Some(access_token) = access_token_guard.as_ref() {
            if !access_token.is_expired(at) {
                let is_valid = self.is_valid_access_token(access_token).await?;
                if is_valid {
                    return Ok(access_token.clone());
                }
            }
        }

        let mut parameters = HashMap::new();
        parameters.insert("client_id", self.app_client_id.as_str());
        parameters.insert("client_secret", self.app_client_secret.as_str());
        parameters.insert("grant_type", "client_credentials");

        let response = self.http_client
            .post(Self::TOKEN_API_BASE_URL)
            .form(&parameters)
            .send()
            .await?
            .json::<dto::access_token::Response>()
            .await?;

        let expires_at = *at + chrono::Duration::seconds(response.expires_in);
        let access_token = AccessToken::new(response.access_token, expires_at);

        *access_token_guard = Some(access_token.clone());

        Ok(access_token)
    }

    async fn is_valid_access_token(&self, access_token: &AccessToken) -> Result<bool, Error> {
        let response = self.http_client
            .get(Self::VALIDATE_API_BASE_URL)
            .bearer_auth(access_token.access_token())
            .send()
            .await?;

        Ok(response.status() == reqwest::StatusCode::OK)
    }
}

mod dto {
    pub mod access_token {
        #[derive(Debug, serde::Deserialize)]
        pub struct Response {
            pub access_token: String,
            pub expires_in: i64,
            // pub token_type: String,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    APIRequestFailed(#[from] reqwest::Error)
}
