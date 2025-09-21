use domain::model::ai_chat_character;
use domain::model::ai_chat_history::AIChatHistory;

pub trait AIChatHistoryRepository {
    fn get(&self, user_id: &str, ai_chat_character_id: &ai_chat_character::Id) -> Result<AIChatHistory, Error>;
    fn set(&self, ai_chat_history: AIChatHistory);
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("entry not found")]
    EntryNotFound(String),
}
