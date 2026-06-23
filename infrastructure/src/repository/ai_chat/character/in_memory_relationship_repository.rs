use application::repository::ai_chat::character::relationship_repository::{
    Error, RelationshipRepository,
};
use dashmap::DashMap;
use domain::model::ai_chat::character::relationship::Relationship;
use domain::model::ai_chat_character::Id;

pub struct InMemoryRelationshipRepository {
    relationships: DashMap<(String, Id), Relationship>,
}

impl Default for InMemoryRelationshipRepository {
    fn default() -> Self {
        Self {
            relationships: DashMap::new(),
        }
    }
}

impl InMemoryRelationshipRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RelationshipRepository for InMemoryRelationshipRepository {
    fn get(&self, user_id: &str, character_id: &Id) -> Result<Relationship, Error> {
        let user_id = user_id.to_string();
        let character_id = character_id.clone();
        self.relationships
            .get(&(user_id.clone(), character_id.clone()))
            .map(|entry| entry.clone())
            .ok_or(Error::EntryNotFound(user_id, character_id))
    }

    fn set(&self, relationship: Relationship) {
        let user_id = relationship.user_id().to_string();
        let character_id = relationship.character_id().clone();
        self.relationships
            .insert((user_id, character_id), relationship);
    }
}
