use application::repository::image_generation_activity_repository::{Error, ImageGenerationActivityRepository};
use dashmap::DashMap;
use domain::model::image_generation_activity::ImageGenerationActivity;

pub struct InMemoryImageGenerationActivityRepository {
    image_generation_activities: DashMap<String, ImageGenerationActivity>,
}

impl InMemoryImageGenerationActivityRepository {
    pub fn new() -> Self {
        Self {
            image_generation_activities: DashMap::new()
        }
    }
}

impl ImageGenerationActivityRepository for InMemoryImageGenerationActivityRepository {
    fn get(&self, user_id: &str) -> Result<ImageGenerationActivity, Error> {
        let user_id = user_id.to_string();
        self.image_generation_activities.get(&user_id)
            .map(|entry| entry.clone())
            .ok_or(Error::EntryNotFound(user_id))
    }

    fn set(&self, ai_chat_activity: ImageGenerationActivity) {
        self.image_generation_activities.insert(ai_chat_activity.user_id().to_string(), ai_chat_activity);
    }
}
