use domain::model::image_generation_activity::ImageGenerationActivity;

pub trait ImageGenerationActivityRepository {
    fn get(&self, user_id: &str) -> Result<ImageGenerationActivity, Error>;
    fn set(&self, ai_chat_activity: ImageGenerationActivity);
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("entry not found")]
    EntryNotFound(String),
}
