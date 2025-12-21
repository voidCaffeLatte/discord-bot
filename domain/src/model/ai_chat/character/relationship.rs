use crate::model::ai_chat_character::Id;
use crate::value_object::ai_chat::character::likability;
use crate::value_object::ai_chat::character::likability::Likability;

#[derive(Clone, Debug)]
pub struct Relationship {
    user_id: String,
    character_id: Id,
    likability: Likability,
}

impl Relationship {
    pub fn new(user_id: String, character_id: Id, likability: Likability) -> Self {
        Self {
            user_id,
            character_id,
            likability,
        }
    }

    pub fn with_key(user_id: String, character_id: Id) -> Self {
        Self::new(user_id, character_id, Likability::default())
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn character_id(&self) -> &Id {
        &self.character_id
    }

    pub fn likability(&self) -> &Likability {
        &self.likability
    }

    pub fn change_likability(&mut self, amount: i32) -> Result<(), likability::Error>{
        let new_value = (self.likability.value() + amount).clamp(Likability::MIN, Likability::MAX);
        let new_likability = Likability::new(new_value)?;
        self.likability = new_likability;
        Ok(())
    }
}
