use serenity::all::{ActionRowComponent, InputText, ResolvedValue};

pub trait ResolvedValueExtension<'a> {
    fn as_integer(&self) -> Option<i64>;
    fn as_string(&self) -> Option<&'a str>;
}

impl<'a> ResolvedValueExtension<'a> for ResolvedValue<'a> {
    fn as_integer(&self) -> Option<i64> {
        match *self {
            ResolvedValue::Integer(value) => Some(value),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<&'a str> {
        match *self {
            ResolvedValue::String(value) => Some(value),
            _ => None,
        }
    }
}

#[allow(dead_code)]
pub trait ActionRowComponentExtension {
    fn custom_id(&self) -> Option<String>;
    fn as_input_text(&self) -> Option<&InputText>;
}

impl ActionRowComponentExtension for ActionRowComponent {
    fn custom_id(&self) -> Option<String> {
        match self {
            ActionRowComponent::InputText(input_text) => Some(input_text.custom_id.clone()),
            _ => None,
        }
    }

    fn as_input_text(&self) -> Option<&InputText> {
        match self {
            ActionRowComponent::InputText(input_text) => Some(input_text),
            _ => None,
        }
    }
}
