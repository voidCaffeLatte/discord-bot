use crate::presentation::command::{CommandFactory, CommandResult, CommandRunner};
use crate::presentation::common::command_option_extractor::CommandOptionExtractor;
use async_trait::async_trait;
use common::fluent_proxy::FluentProxy;
use fluent::fluent_args;
use rand::prelude::IndexedRandom;
use serenity::all::{CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption, CreateInteractionResponseFollowup};
use std::sync::Arc;

pub struct Choices {
    fluent_proxy: Arc<FluentProxy>,
}

impl Choices {
    const MAX_COUNT: u64 = 10;

    pub fn new(
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            fluent_proxy
        }
    }
}

#[async_trait]
impl CommandRunner for Choices {
    async fn run(&self, context: &Context, interaction: &CommandInteraction) -> CommandResult {
        interaction.defer(&context.http).await?;

        let options = interaction.data.options();
        let command_option_extractor = CommandOptionExtractor::new(&options);

        let choices = (1..=Self::MAX_COUNT)
            .map(|index| format!("choice-{}", index))
            .filter_map(|key| command_option_extractor.get_string(&key).ok())
            .collect::<Vec<_>>();
        let count: usize = command_option_extractor.get_integer("count").unwrap_or(1).try_into()?;

        let selected_choices: Vec<_> = choices.sample(&mut rand::rng(), count).collect();

        let original_choices_message = choices
            .iter()
            .map(|choice| {
                let fluent_args = fluent_args![
                    "choice" => *choice
                ];
                self.fluent_proxy.get_message("choices--response--choice", Some(&fluent_args))
            })
            .collect::<Vec<_>>()
            .join("\n");

        let selected_choices_message = selected_choices
            .iter()
            .map(|choice| {
                let fluent_args = fluent_args![
                    "choice" => **choice
                ];
                self.fluent_proxy.get_message("choices--response--choice", Some(&fluent_args))
            })
            .collect::<Vec<_>>()
            .join("\n");

        let fluent_args = fluent_args![
            "original-choices" => original_choices_message,
            "selected-choices" => selected_choices_message
        ];
        let response_message = self.fluent_proxy.get_message("choices--response--body", Some(&fluent_args));

        interaction.create_followup(
            &context.http,
            CreateInteractionResponseFollowup::new().content(response_message),
        ).await?;

        Ok(())
    }
}

pub struct Factory {
    fluent_proxy: Arc<FluentProxy>,
}

impl Factory {
    pub fn new(
        fluent_proxy: Arc<FluentProxy>
    ) -> Self {
        Self {
            fluent_proxy
        }
    }
}

impl CommandFactory for Factory {
    fn command_name(&self) -> String {
        "choices".to_string()
    }

    fn command_specification(&self) -> CreateCommand {
        let mut command_options: Vec<CreateCommandOption> = (1..=Choices::MAX_COUNT).map(|index| {
            let fluent_args = fluent_args![
                "index" => index,
            ];
            let command_option_description = self.fluent_proxy.get_message("choices--command-option--choice--description", Some(&fluent_args));

            let command_option = CreateCommandOption::new(
                CommandOptionType::String,
                format!("choice-{}", index),
                command_option_description);

            command_option
                .required(index <= 2)
                .min_length(1)
                .max_length(100)
        }).collect();

        let command_description = self.fluent_proxy.get_message("choices--command-option--count--description", None);
        command_options.push(
            CreateCommandOption::new(CommandOptionType::Integer, "count", command_description)
                .required(false)
                .min_int_value(1)
                .max_int_value(Choices::MAX_COUNT - 1));

        let command_description = self.fluent_proxy.get_message("choices--command--description", None);
        CreateCommand::new(self.command_name())
            .description(command_description)
            .set_options(command_options)
    }

    fn create(&self) -> Box<dyn CommandRunner + Send + Sync> {
        Box::new(Choices::new(self.fluent_proxy.clone()))
    }
}
