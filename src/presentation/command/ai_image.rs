use crate::presentation::command::{CommandFactory, CommandResult, CommandRunner};
use crate::presentation::common::command_option_extractor::CommandOptionExtractor;
use application::use_case::generate_ai_image_use_case::{GenerateAIImageUseCase, UseCaseError};
use async_trait::async_trait;
use chrono;
use chrono::Utc;
use common::fluent_proxy::FluentProxy;
use domain::model::image_generation_activity::ImageGenerationActivity;
use fluent::fluent_args;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateAttachment, CreateCommand,
    CreateCommandOption, CreateInteractionResponseFollowup,
};
use std::sync::Arc;

pub struct AIImage {
    generate_ai_image_use_case: Arc<GenerateAIImageUseCase>,
    fluent_proxy: Arc<FluentProxy>,
}

impl AIImage {
    pub fn new(
        generate_ai_image_use_case: Arc<GenerateAIImageUseCase>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            generate_ai_image_use_case,
            fluent_proxy,
        }
    }
}

#[async_trait]
impl CommandRunner for AIImage {
    async fn run(&self, context: &Context, interaction: &CommandInteraction) -> CommandResult {
        interaction.defer(&context.http).await?;

        let now = Utc::now();

        let user_id = interaction.user.id.to_string();

        let options = interaction.data.options();
        let command_option_extarctor = CommandOptionExtractor::new(&options);
        let prompt = command_option_extarctor.get_string("prompt")?;

        let use_case_result = match self.generate_ai_image_use_case
            .run(&user_id, prompt, &now)
            .await {
            Ok(result) => result,
            Err(UseCaseError::GenerationCountExceeded) => {
                let error_message = self.fluent_proxy.get_message("ai-image--error--execution-limit-exceeded", None);
                interaction.create_followup(&context.http, CreateInteractionResponseFollowup::new().content(error_message)).await?;
                return Ok(());
            }
            Err(error) => return Err(error.into())
        };

        let fluent_args = fluent_args![
            "prompt" => prompt,
        ];
        let response_message = self.fluent_proxy.get_message("ai-image--response--body", Some(&fluent_args));

        let followup_response = CreateInteractionResponseFollowup::new()
            .content(response_message)
            .add_file(CreateAttachment::bytes(use_case_result.image_bytes, "response.png"));

        interaction.create_followup(&context.http, followup_response).await?;

        Ok(())
    }
}

pub struct Factory {
    generate_ai_image_use_case: Arc<GenerateAIImageUseCase>,
    fluent_proxy: Arc<FluentProxy>,
}

impl Factory {
    pub fn new(
        generate_ai_image_use_case: Arc<GenerateAIImageUseCase>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            generate_ai_image_use_case,
            fluent_proxy,
        }
    }
}

impl CommandFactory for Factory {
    fn command_name(&self) -> String {
        "ai-image".to_string()
    }

    fn command_specification(&self) -> CreateCommand
    {
        let prompt_option = CreateCommandOption::new(
            CommandOptionType::String,
            "prompt",
            self.fluent_proxy.get_message("ai-image--command-option--prompt--description", None),
        )
            .required(true)
            .min_length(1)
            .max_length(250);

        let fluent_args = fluent_args![
            "execution-limit" => ImageGenerationActivity::MAX_COUNT_PER_DAY.to_string()
        ];
        let command_description = self.fluent_proxy.get_message("ai-image--command--description", Some(&fluent_args));
        CreateCommand::new(self.command_name())
            .description(command_description)
            .add_option(prompt_option)
    }

    fn create(&self) -> Box<dyn CommandRunner + Send + Sync> {
        Box::new(AIImage::new(
            self.generate_ai_image_use_case.clone(),
            self.fluent_proxy.clone(),
        ))
    }
}
