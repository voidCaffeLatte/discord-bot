use domain::model::ai_chat_activity::AIChatActivity;

pub trait AIChatActivityRepository {
    fn get(&self, user_id: &str) -> Result<AIChatActivity, Error>;
    fn set(&self, ai_chat_activity: AIChatActivity);
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("entry not found")]
    EntryNotFound(String),
}
