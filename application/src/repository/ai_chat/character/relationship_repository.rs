use domain::model::ai_chat::character::relationship::Relationship;
use domain::model::ai_chat_character::Id;

pub trait RelationshipRepository {
    fn get(&self, user_id: &str, character_id: &Id) -> Result<Relationship, Error>;
    fn set(&self, relationship: Relationship);
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("entry not found")]
    EntryNotFound(String, Id),
}
