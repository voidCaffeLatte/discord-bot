use crate::model::ai_chat_character;

#[derive(Clone)]
pub struct AIChatHistory {
    user_id: String,
    ai_chat_character_id: ai_chat_character::Id,
    chat_entries: Vec<ChatEntry>,
}

impl AIChatHistory {
    const MAX_ENTRY_COUNT: u32 = 2;

    pub fn with_key(user_id: String, ai_chat_character_id: ai_chat_character::Id) -> Self {
        Self {
            user_id,
            ai_chat_character_id,
            chat_entries: Vec::default(),
        }
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn ai_chat_character_id(&self) -> &ai_chat_character::Id {
        &self.ai_chat_character_id
    }

    pub fn chat_entries(&self) -> &[ChatEntry] {
        &self.chat_entries
    }

    pub fn add_chat_entry(&mut self, chat_entry: ChatEntry) {
        self.chat_entries.push(chat_entry);
        if self.chat_entries.len() as u32 > Self::MAX_ENTRY_COUNT
        {
            self.chat_entries.remove(0);
        }
    }
}

#[derive(Clone)]
pub struct ChatEntry {
    pub request: String,
    pub response: String,
}

