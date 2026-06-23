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

    pub fn change_likability(&mut self, amount: i32) -> Result<(), likability::Error> {
        let new_value = (self.likability.value() + amount).clamp(Likability::MIN, Likability::MAX);
        let new_likability = Likability::new(new_value)?;
        self.likability = new_likability;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn with_key_should_create_with_base_likability() {
        let relationship = Relationship::with_key("user1".to_string(), Id(1));
        assert_eq!(relationship.likability().value(), Likability::BASE);
    }

    #[rstest]
    #[case(0, 10, 10)]
    #[case(0, -10, -10)]
    #[case(50, 0, 50)]
    #[case(90, 20, 100)] // clamped to MAX
    #[case(-90, -20, -100)] // clamped to MIN
    #[case(0, 200, 100)] // large positive clamped
    #[case(0, -200, -100)] // large negative clamped
    fn change_likability_should_update_and_clamp(
        #[case] initial: i32,
        #[case] change: i32,
        #[case] expected: i32,
    ) {
        let mut rel = Relationship::new(
            "user1".to_string(),
            Id(1),
            Likability::new(initial).unwrap(),
        );
        rel.change_likability(change).unwrap();
        assert_eq!(rel.likability().value(), expected);
    }
}
