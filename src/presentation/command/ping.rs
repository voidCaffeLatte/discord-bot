use crate::presentation::command::{CommandFactory, CommandResult, CommandRunner};
use async_trait::async_trait;
use common::fluent_proxy::FluentProxy;
use serenity::all::{
    CommandInteraction, Context, CreateCommand, CreateInteractionResponseFollowup,
};
use std::sync::Arc;

pub struct Ping {
    fluent_proxy: Arc<FluentProxy>,
}

impl Ping {
    pub fn new(fluent_proxy: Arc<FluentProxy>) -> Self {
        Self { fluent_proxy }
    }
}

#[async_trait]
impl CommandRunner for Ping {
    async fn run(&self, context: &Context, interaction: &CommandInteraction) -> CommandResult {
        interaction.defer(&context.http).await?;

        let response_message = self.fluent_proxy.get_message("ping--response--body", None);
        let followup = CreateInteractionResponseFollowup::new().content(response_message);
        interaction.create_followup(&context.http, followup).await?;

        Ok(())
    }
}

pub struct Factory {
    fluent_proxy: Arc<FluentProxy>,
}

impl Factory {
    pub fn new(fluent_proxy: Arc<FluentProxy>) -> Self {
        Self { fluent_proxy }
    }
}

impl CommandFactory for Factory {
    fn command_name(&self) -> String {
        "ping".to_string()
    }

    fn command_specification(&self) -> CreateCommand {
        let command_description = self
            .fluent_proxy
            .get_message("ping--command--description", None);
        CreateCommand::new(self.command_name()).description(command_description)
    }

    fn create(&self) -> Box<dyn CommandRunner + Send + Sync> {
        Box::new(Ping::new(self.fluent_proxy.clone()))
    }
}
