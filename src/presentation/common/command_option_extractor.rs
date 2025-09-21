use crate::presentation::common::serenity_extension::ResolvedValueExtension;
use serenity::all::ResolvedOption;
use std::collections::HashMap;

pub struct CommandOptionExtractor<'a> {
    map: HashMap<String, &'a ResolvedOption<'a>>,
}

impl<'a> CommandOptionExtractor<'a> {
    pub fn new(options: &'a [ResolvedOption<'a>]) -> Self {
        let map = options
            .iter()
            .map(|opt| (opt.name.to_string(), opt))
            .collect();

        Self { map }
    }

    pub fn get_string(&self, name: &str) -> Result<&str, Error> {
        self.map
            .get(name)
            .ok_or(Error::OptionNotFound(name.to_string()))?
            .value
            .as_string()
            .ok_or(Error::InvalidType {
                option_name: name.to_string(),
                expected: "string",
            })
    }

    pub fn get_integer(&self, name: &str) -> Result<i64, Error> {
        self.map
            .get(name)
            .ok_or(Error::OptionNotFound(name.to_string()))?
            .value
            .as_integer()
            .ok_or(Error::InvalidType {
                option_name: name.to_string(),
                expected: "integer",
            })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("option is not found")]
    OptionNotFound(String),

    #[error("invalid type")]
    InvalidType { option_name: String, expected: &'static str },
}
