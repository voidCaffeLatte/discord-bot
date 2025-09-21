use domain::model::ai_chat_character;
use domain::model::ai_chat_character::AIChatCharacter;

pub trait AIChatCharacterRepository {
    fn get(&self, ids: &[ai_chat_character::Id]) -> Result<Vec<&AIChatCharacter>, Error>;
    fn get_all(&self) -> Vec<&AIChatCharacter>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("character is not found")]
    CharacterNotFound(ai_chat_character::Id),
}
