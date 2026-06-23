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
        if self.chat_entries.len() as u32 > Self::MAX_ENTRY_COUNT {
            self.chat_entries.remove(0);
        }
    }
}

#[derive(Clone)]
pub struct ChatEntry {
    pub request: String,
    pub response: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ai_chat_character::Id;
    use rstest::{fixture, rstest};

    #[fixture]
    fn history() -> AIChatHistory {
        AIChatHistory::with_key("user1".to_string(), Id(1))
    }

    fn entry(n: u32) -> ChatEntry {
        ChatEntry {
            request: format!("req{n}"),
            response: format!("res{n}"),
        }
    }

    // Adding entries up to MAX_ENTRY_COUNT keeps all of them
    #[rstest]
    fn add_chat_entry_should_keep_entries_up_to_max(mut history: AIChatHistory) {
        history.add_chat_entry(entry(1));
        history.add_chat_entry(entry(2));

        assert_eq!(history.chat_entries().len(), 2);
        assert_eq!(history.chat_entries()[0].request, "req1");
        assert_eq!(history.chat_entries()[1].request, "req2");
    }

    // Exceeding MAX_ENTRY_COUNT drops the oldest, keeping the most recent entries
    #[rstest]
    #[case(3, &["req2", "req3"])]
    #[case(4, &["req3", "req4"])]
    fn add_chat_entry_should_drop_oldest_on_overflow(
        mut history: AIChatHistory,
        #[case] total: u32,
        #[case] expected_requests: &[&str],
    ) {
        for i in 1..=total {
            history.add_chat_entry(entry(i));
        }

        assert_eq!(history.chat_entries().len(), expected_requests.len());
        for (entry, expected) in history.chat_entries().iter().zip(expected_requests) {
            assert_eq!(entry.request, *expected);
        }
    }
}
