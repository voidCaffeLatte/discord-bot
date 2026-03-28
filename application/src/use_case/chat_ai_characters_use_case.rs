use crate::gateway::ai_text_generation_gateway::{AITextGenerationGateway, GatewayError, Message, Role};
use crate::repository::ai_chat_activity_repository::AIChatActivityRepository;
use crate::repository::ai_chat_character_repository::AIChatCharacterRepository;
use crate::repository::{ai_chat_activity_repository, ai_chat_character_repository};
use chrono::{DateTime, Utc};
use common::fluent_proxy::FluentProxy;
use common::unique_collection::UniqueCollection;
use domain::model::ai_chat_activity::AIChatActivity;
use domain::model::ai_chat_character;
use domain::value_object::ai_text::AIText;
use fluent::fluent_args;
use std::sync::Arc;

pub struct ChatAICharactersUseCase {
    ai_text_generation_gateway: Arc<dyn AITextGenerationGateway + Send + Sync>,
    ai_chat_character_repository: Arc<dyn AIChatCharacterRepository + Send + Sync>,
    ai_chat_activity_repository: Arc<dyn AIChatActivityRepository + Send + Sync>,
    fluent_proxy: Arc<FluentProxy>,
}

impl ChatAICharactersUseCase {
    pub const MAX_CHAT_COUNT_PER_USER: u32 = 10;

    pub fn new(
        ai_text_generation_gateway: Arc<dyn AITextGenerationGateway + Send + Sync>,
        ai_chat_character_repository: Arc<dyn AIChatCharacterRepository + Send + Sync>,
        ai_chat_activity_repository: Arc<dyn AIChatActivityRepository + Send + Sync>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            ai_text_generation_gateway,
            ai_chat_character_repository,
            ai_chat_activity_repository,
            fluent_proxy,
        }
    }

    pub async fn run(
        &self,
        user_id: &str,
        ai_chat_character_ids: &UniqueCollection<ai_chat_character::Id>,
        message: &str,
        at: &DateTime<Utc>,
    ) -> Result<UseCaseResult, Error> {
        let ai_chat_characters = match self.ai_chat_character_repository.get(ai_chat_character_ids.values()) {
            Ok(ai_chata_characters) => ai_chata_characters,
            Err(ai_chat_character_repository::Error::CharacterNotFound(_)) => return Err(Error::CharacterNotFound)
        };

        if ai_chat_characters.len() < 2 {
            return Err(Error::CharacterCountNotEnough);
        }

        let mut ai_chat_activity = match self.ai_chat_activity_repository.get(user_id) {
            Ok(ai_chat_activity) => ai_chat_activity,
            Err(ai_chat_activity_repository::Error::EntryNotFound(_)) => AIChatActivity::with_key(user_id.to_string())
        };
        let current_chat_count = ai_chat_activity.current_chat_count(at);
        if current_chat_count >= Self::MAX_CHAT_COUNT_PER_USER {
            return Err(Error::ChatCountExceeded);
        }

        let characters_message = ai_chat_characters
            .iter()
            .map(|ai_chat_character| {
                let fluent_args = fluent_args![
                    "name" => ai_chat_character.character_name(),
                    "title" => ai_chat_character.title(),
                    "characteristics" => ai_chat_character.characteristics().join(", "),
                ];
                self.fluent_proxy.get_message("ai-conversation--system-prompt--character", Some(&fluent_args))
            })
            .collect::<Vec<_>>()
            .join(", ");

        let fluent_args = fluent_args![
            "characters-message" => characters_message,
        ];
        let system_prompt = self.fluent_proxy.get_message("ai-conversation--system-prompt--body", Some(&fluent_args));

        let fluent_args = fluent_args![
            "theme" => message,
        ];
        let user_prompt = self.fluent_proxy.get_message("ai-conversation--user-prompt", Some(&fluent_args));
        let messages = [Message::new(Role::User, user_prompt.to_string())];

        let result = self.ai_text_generation_gateway.generate_text(&messages, &system_prompt).await?;

        ai_chat_activity.increment_chat_count(at);
        self.ai_chat_activity_repository.set(ai_chat_activity);

        Ok(UseCaseResult { ai_text: result })
    }
}

pub struct UseCaseResult {
    pub ai_text: AIText,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("chat count is exceeded")]
    ChatCountExceeded,

    #[error("character count is not enough")]
    CharacterCountNotEnough,

    #[error("character is not found")]
    CharacterNotFound,

    #[error(transparent)]
    RequestError(#[from] GatewayError),
}
