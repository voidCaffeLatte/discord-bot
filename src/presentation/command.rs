use async_trait::async_trait;
use serenity::all::{CommandInteraction, Context};
use serenity::builder::CreateCommand;

pub mod ping;
pub mod choices;
pub mod ai_chat;
pub mod ai_conversation;
pub mod random_you_tube_video;
pub mod ai_image;
pub mod random_twitch_clip;

#[async_trait]
pub trait CommandRunner {
    async fn run(&self, context: &Context, interaction: &CommandInteraction) -> CommandResult;
}

pub trait CommandFactory {
    fn command_name(&self) -> String;

    fn command_specification(&self) -> CreateCommand;

    fn create(&self) -> Box<dyn CommandRunner + Send + Sync>;
}

pub type CommandError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type CommandResult = Result<(), CommandError>;
