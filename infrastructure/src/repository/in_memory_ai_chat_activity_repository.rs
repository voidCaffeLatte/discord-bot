use application::repository::ai_chat_activity_repository::{AIChatActivityRepository, Error};
use dashmap::DashMap;
use domain::model::ai_chat_activity::AIChatActivity;

pub struct InMemoryAIChatActivityRepository {
    ai_chat_activities: DashMap<String, AIChatActivity>,
}

impl InMemoryAIChatActivityRepository {
    pub fn new() -> Self {
        Self { ai_chat_activities: DashMap::new() }
    }
}

impl AIChatActivityRepository for InMemoryAIChatActivityRepository {
    fn get(&self, user_id: &str) -> Result<AIChatActivity, Error> {
        let user_id = user_id.to_string();
        self.ai_chat_activities.get(&user_id)
            .map(|entry| entry.clone())
            .ok_or(Error::EntryNotFound(user_id))
    }

    fn set(&self, ai_chat_activity: AIChatActivity) {
        self.ai_chat_activities.insert(ai_chat_activity.user_id().to_string(), ai_chat_activity);
    }
}
