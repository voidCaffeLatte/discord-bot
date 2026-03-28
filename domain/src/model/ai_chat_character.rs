#[derive(Clone, Debug)]
pub struct AIChatCharacter {
    id: Id,
    title: String,
    character_name: String,
    characteristics: Vec<String>,
}

impl AIChatCharacter {
    pub fn new(
        id: Id,
        title: String,
        character_name: String,
        characteristics: Vec<String>,
    ) -> Self {
        Self {
            id,
            title,
            character_name,
            characteristics,
        }
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn character_name(&self) -> &str {
        &self.character_name
    }

    pub fn display_name(&self) -> String {
        format!("{} - {}", self.character_name, self.title)
    }

    pub fn characteristics(&self) -> &[String] {
        &self.characteristics
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Id(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_should_format_as_character_name_dash_title() {
        let character = AIChatCharacter::new(
            Id(1),
            "title".to_string(),
            "character_name".to_string(),
            vec![],
        );
        assert_eq!(character.display_name(), "character_name - title");
    }
}
