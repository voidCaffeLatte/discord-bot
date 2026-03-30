use application;
use crate::gateway::twitch;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use domain::value_object::twitch::user::User;
use domain::value_object::twitch::user_login_id::UserLoginId;
use application::gateway::twitch::user_gateway;

pub struct UserGateway {
    http_client: reqwest::Client,
    twitch_app_client_id: String,
    twitch_access_token_gateway: Arc<twitch::access_token_gateway::AccessTokenGateway>,
}

impl UserGateway {
    const BASE_URL: &'static str = "https://api.twitch.tv/helix/users";

    pub fn new(
        http_client: reqwest::Client,
        twitch_app_client_id: String,
        twitch_access_token_gateway: Arc<twitch::access_token_gateway::AccessTokenGateway>,
    ) -> Self {
        Self {
            http_client,
            twitch_app_client_id,
            twitch_access_token_gateway,
        }
    }
}

#[async_trait]
impl user_gateway::UserGateway for UserGateway {
    async fn get_by_user_login_id(
        &self,
        user_login_id: &UserLoginId,
        at: &DateTime<Utc>,
    ) -> Result<User, user_gateway::Error> {
        let access_token = self.twitch_access_token_gateway.get_access_token(at).await
            .map_err(|error| user_gateway::Error::RetrievalFailed(error.into()))?;

        let mut query_parameters = HashMap::new();
        query_parameters.insert("login", user_login_id.id());

        let response = self.http_client
            .get(Self::BASE_URL)
            .bearer_auth(access_token.access_token())
            .header("Client-Id", &self.twitch_app_client_id)
            .query(&query_parameters)
            .send()
            .await
            .map_err(|e| user_gateway::Error::RetrievalFailed(e.into()))?
            .error_for_status()
            .map_err(|e| user_gateway::Error::RetrievalFailed(e.into()))?
            .json::<dto::Response>()
            .await
            .map_err(|error| user_gateway::Error::InvalidResponse(error.into()))?;

        let user_data = response.data.as_ref()
            .and_then(|data| data.first())
            .ok_or(user_gateway::Error::UserNotFound)?;
        let id = user_data.id.as_ref()
            .ok_or_else(|| user_gateway::Error::InvalidResponse(anyhow::anyhow!("user id field is missing in response")))?;

        Ok(User::new(id.clone()))
    }
}


mod dto {
    #[derive(Debug, serde::Deserialize)]
    pub struct Response {
        pub data: Option<Vec<Data>>,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct Data {
        pub id: Option<String>,
        // pub login: String,
        // pub display_name: String,
    }
}

