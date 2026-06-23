use application::repository::ai_chat_character_repository::{AIChatCharacterRepository, Error};
use domain::model::ai_chat_character;
use domain::model::ai_chat_character::AIChatCharacter;
use std::collections::HashMap;
use std::path::Path;
use std::{fs, io};

pub struct InMemoryAIChatCharacterRepository {
    ai_chat_characters: HashMap<ai_chat_character::Id, AIChatCharacter>,
}

impl InMemoryAIChatCharacterRepository {
    pub fn try_new(file_path: &Path) -> Result<Self, InitializationError> {
        let character_file_text = fs::read_to_string(file_path)?;
        let content: dto::Content = toml::from_str(&character_file_text)?;

        let ai_chat_characters = content
            .characters
            .into_iter()
            .enumerate()
            .map(|(index, character)| {
                let ai_chat_character = AIChatCharacter::new(
                    ai_chat_character::Id(index as u32),
                    character.title,
                    character.name,
                    character.characteristics,
                );
                (ai_chat_character.id().clone(), ai_chat_character)
            })
            .collect::<HashMap<_, _>>();

        Ok(Self { ai_chat_characters })
    }
}

impl AIChatCharacterRepository for InMemoryAIChatCharacterRepository {
    fn get(&self, ids: &[ai_chat_character::Id]) -> Result<Vec<&AIChatCharacter>, Error> {
        ids.iter()
            .map(|id| {
                self.ai_chat_characters
                    .get(id)
                    .ok_or(Error::CharacterNotFound(id.clone()))
            })
            .collect()
    }

    fn get_all(&self) -> Vec<&AIChatCharacter> {
        self.ai_chat_characters.values().collect()
    }
}

mod dto {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct Content {
        pub characters: Vec<Character>,
    }

    #[derive(Deserialize)]
    pub struct Character {
        pub name: String,
        pub title: String,
        pub characteristics: Vec<String>,
    }
}

#[derive(thiserror::Error, Debug)]
pub enum InitializationError {
    #[error("file loading is failed")]
    FileLoadingFailed(#[source] io::Error),

    #[error("invalid file format")]
    InvalidFileFormat(#[source] toml::de::Error),
}

impl From<io::Error> for InitializationError {
    fn from(value: io::Error) -> Self {
        InitializationError::FileLoadingFailed(value)
    }
}

impl From<toml::de::Error> for InitializationError {
    fn from(value: toml::de::Error) -> Self {
        InitializationError::InvalidFileFormat(value)
    }
}
