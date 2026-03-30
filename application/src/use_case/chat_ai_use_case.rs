use crate::gateway::ai_structured_text_generation_gateway::AIStructuredTextGenerationGateway;
use crate::gateway::ai_text_generation_gateway::{Message, Role};
use crate::repository::ai_chat_activity_repository::AIChatActivityRepository;
use crate::repository::ai_chat_character_repository::AIChatCharacterRepository;
use crate::repository::ai_chat_history_repository::AIChatHistoryRepository;
use crate::repository::{ai_chat_activity_repository, ai_chat_character_repository, ai_chat_history_repository};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use fluent::fluent_args;
use common::fluent_proxy::FluentProxy;
use domain::model::ai_chat::character::relationship::Relationship;
use domain::model::ai_chat_activity::AIChatActivity;
use domain::model::ai_chat_character;
use domain::model::ai_chat_history::{AIChatHistory, ChatEntry};
use domain::value_object::ai_chat::character::likability::Likability;
use domain::value_object::ai_text::AIText;
use crate::repository::ai_chat::character::relationship_repository;
use crate::repository::ai_chat::character::relationship_repository::RelationshipRepository;

pub struct ChatAIUseCase {
    ai_text_generation_gateway: Arc<dyn AIStructuredTextGenerationGateway<dto::Response> + Send + Sync>,
    ai_chat_character_repository: Arc<dyn AIChatCharacterRepository + Send + Sync>,
    ai_chat_history_repository: Arc<dyn AIChatHistoryRepository + Send + Sync>,
    ai_chat_activity_repository: Arc<dyn AIChatActivityRepository + Send + Sync>,
    ai_chat_character_relationship_repository: Arc<dyn RelationshipRepository + Send + Sync>,
    fluent_proxy: Arc<FluentProxy>,
}

impl ChatAIUseCase {
    pub const MAX_CHAT_COUNT_PER_USER: u32 = 10;

    pub fn new(
        ai_text_generation_gateway: Arc<dyn AIStructuredTextGenerationGateway<dto::Response> + Send + Sync>,
        ai_chat_character_repository: Arc<dyn AIChatCharacterRepository + Send + Sync>,
        ai_chat_history_repository: Arc<dyn AIChatHistoryRepository + Send + Sync>,
        ai_chat_activity_repository: Arc<dyn AIChatActivityRepository + Send + Sync>,
        ai_chat_character_relationship_repository: Arc<dyn RelationshipRepository + Send + Sync>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            ai_text_generation_gateway,
            ai_chat_character_repository,
            ai_chat_history_repository,
            ai_chat_activity_repository,
            ai_chat_character_relationship_repository,
            fluent_proxy,
        }
    }

    pub async fn run(
        &self,
        user_id: &str,
        user_name: &str,
        ai_chat_character_id: ai_chat_character::Id,
        message: &str,
        at: &DateTime<Utc>,
    ) -> Result<UseCaseResult, UseCaseError> {
        let ai_chat_characters = match self.ai_chat_character_repository.get(&[ai_chat_character_id]) {
            Ok(ai_chat_characters) => ai_chat_characters,
            Err(ai_chat_character_repository::Error::CharacterNotFound(_)) => return Err(UseCaseError::CharacterNotFound),
        };
        let ai_chat_character = ai_chat_characters.first().ok_or(UseCaseError::CharacterNotFound)?;

        let mut ai_chat_activity = match self.ai_chat_activity_repository.get(user_id) {
            Ok(ai_chat_activity) => ai_chat_activity,
            Err(ai_chat_activity_repository::Error::EntryNotFound(_)) => AIChatActivity::with_key(user_id.to_string())
        };
        let current_chat_count = ai_chat_activity.current_chat_count(at);
        if current_chat_count >= Self::MAX_CHAT_COUNT_PER_USER {
            return Err(UseCaseError::ChatCountExceeded);
        }

        let mut ai_chat_history = match self.ai_chat_history_repository.get(user_id, ai_chat_character.id()) {
            Ok(ai_chat_history) => ai_chat_history,
            Err(ai_chat_history_repository::Error::EntryNotFound(_)) => AIChatHistory::with_key(user_id.to_string(), ai_chat_character.id().clone())
        };

        let history_messages: Vec<_> = ai_chat_history
            .chat_entries()
            .iter()
            .flat_map(|entry| {
                [
                    Message::new(Role::User, entry.request.clone()),
                    Message::new(Role::Model, entry.response.clone()),
                ]
            })
            .collect();

        let mut relationship = match self.ai_chat_character_relationship_repository.get(user_id, ai_chat_character.id()) {
            Ok(relationship) => relationship,
            Err(relationship_repository::Error::EntryNotFound(_, _)) => Relationship::with_key(user_id.to_string(), ai_chat_character.id().clone())
        };

        let current_datetime = at.with_timezone(&chrono_tz::Asia::Tokyo).format("%Y-%m-%d %H:%M %:z").to_string();
        let fluent_args = fluent_args![
            "name" => user_name,
            "message" => message,
            "likability" => relationship.likability().value(),
            "current-datetime" => current_datetime,
        ];
        let user_prompt = self.fluent_proxy.get_message("ai-chat--user-prompt--body", Some(&fluent_args));

        let messages = history_messages
            .into_iter()
            .chain(std::iter::once(Message::new(Role::User, user_prompt.to_string())))
            .collect::<Vec<_>>();

        let fluent_args = fluent_args![
            "name" => ai_chat_character.character_name(),
            "title" => ai_chat_character.title(),
            "characteristics" => ai_chat_character.characteristics().join(", "),
            "base-likability" => Likability::BASE,
            "min-likability" => Likability::MIN,
            "max-likability" => Likability::MAX
        ];
        let system_prompt = self.fluent_proxy.get_message("ai-chat--system-prompt--body", Some(&fluent_args));

        let result = self.ai_text_generation_gateway.generate_structured_text(&messages, &system_prompt).await
            .map_err(|error| UseCaseError::RequestError(error.into()))?;
        let (response, web_references) = result.into_parts();

        relationship.change_likability(response.likability_change)
            .map_err(|error| UseCaseError::InvalidLikability(error.into()))?;
        self.ai_chat_character_relationship_repository.set(relationship);

        ai_chat_activity.increment_chat_count(at);
        self.ai_chat_activity_repository.set(ai_chat_activity);

        ai_chat_history.add_chat_entry(ChatEntry { request: message.to_string(), response: response.message.clone() });
        self.ai_chat_history_repository.set(ai_chat_history);

        let ai_text = AIText::new(response.message, web_references);

        Ok(UseCaseResult { ai_character_name: ai_chat_character.display_name().to_string(), ai_text })
    }
}

pub struct UseCaseResult {
    pub ai_character_name: String,
    pub ai_text: AIText,
}

#[derive(thiserror::Error, Debug)]
pub enum UseCaseError {
    #[error("chat count is exceeded")]
    ChatCountExceeded,

    #[error("character is not found")]
    CharacterNotFound,

    #[error("invalid likability")]
    InvalidLikability(#[source] anyhow::Error),

    #[error("failed to generate AI chat response")]
    RequestError(#[source] anyhow::Error),
}

mod dto {
    use schemars::{JsonSchema};

    #[derive(Debug, serde::Deserialize, serde::Serialize, JsonSchema)]
    pub struct Response {
        #[schemars(description = "Response message by ai chat character", example = "Hello!")]
        pub message: String,

        #[schemars(description = "Amount of change in likability based on messages from users")]
        pub likability_change: i32,
    }
}
