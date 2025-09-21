use std::borrow::Cow;
use fluent::bundle::FluentBundle;
use fluent::{FluentArgs, FluentResource};
use intl_memoizer::concurrent::IntlLangMemoizer;
use tracing::warn;

pub struct FluentProxy {
    fluent_bundle: FluentBundle<FluentResource, IntlLangMemoizer>,
}

impl FluentProxy {
    pub fn new(fluent_bundle: FluentBundle<FluentResource, IntlLangMemoizer>) -> Self {
        Self {
            fluent_bundle
        }
    }

    pub fn get_message<'a>(&'a self, message_id: &'a str, args: Option<&FluentArgs>) -> Cow<'a, str> {
        let message = self.fluent_bundle.get_message(message_id);

        let Some(message) = message else {
            warn!("Fluent message is not found in bundle: {}", message_id);
            return Cow::Borrowed(message_id);
        };

        let Some(pattern) = message.value() else {
            warn!("Fluent message has no value: {}", message_id);
            return Cow::Borrowed(message_id);
        };

        let mut errors = vec![];
        let result = self.fluent_bundle.format_pattern(pattern, args, &mut errors);

        for error in &errors {
            warn!("Fluent error occurred: {}, {}", message_id, error);
        }

        result
    }
}
