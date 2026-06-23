use application::repository::ai_chat_history_repository::{AIChatHistoryRepository, Error};
use dashmap::DashMap;
use domain::model::ai_chat_character;
use domain::model::ai_chat_history::AIChatHistory;

pub struct InMemoryAIChatHistoryRepository {
    ai_chat_histories: DashMap<(String, ai_chat_character::Id), AIChatHistory>,
}

impl Default for InMemoryAIChatHistoryRepository {
    fn default() -> Self {
        Self {
            ai_chat_histories: DashMap::new(),
        }
    }
}

impl InMemoryAIChatHistoryRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AIChatHistoryRepository for InMemoryAIChatHistoryRepository {
    fn get(
        &self,
        user_id: &str,
        ai_chat_character_id: &ai_chat_character::Id,
    ) -> Result<AIChatHistory, Error> {
        self.ai_chat_histories
            .get(&(user_id.to_string(), ai_chat_character_id.clone()))
            .map(|entry| entry.clone())
            .ok_or(Error::EntryNotFound(user_id.to_string()))
    }

    fn set(&self, ai_chat_history: AIChatHistory) {
        let key = (
            ai_chat_history.user_id().to_string(),
            ai_chat_history.ai_chat_character_id().clone(),
        );
        self.ai_chat_histories.insert(key, ai_chat_history);
    }
}
