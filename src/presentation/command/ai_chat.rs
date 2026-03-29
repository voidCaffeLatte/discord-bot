use crate::presentation::command::{CommandFactory, CommandResult, CommandRunner};
use crate::presentation::common::command_option_extractor::CommandOptionExtractor;
use application::repository::ai_chat_character_repository::AIChatCharacterRepository;
use application::use_case;
use application::use_case::chat_ai_use_case::ChatAIUseCase;
use async_trait::async_trait;
use chrono::Utc;
use common::fluent_proxy::FluentProxy;
use domain::model::ai_chat_character;
use fluent::fluent_args;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateAttachment, CreateCommand,
    CreateCommandOption, CreateInteractionResponseFollowup,
};
use std::sync::Arc;

const MAX_MESSAGE_LENGTH: usize = 1950;

pub struct AIChat {
    chat_ai_use_case: Arc<ChatAIUseCase>,
    fluent_proxy: Arc<FluentProxy>,
}

impl AIChat {
    pub fn new(
        chat_ai_use_case: Arc<ChatAIUseCase>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            chat_ai_use_case,
            fluent_proxy,
        }
    }
}

#[async_trait]
impl CommandRunner for AIChat {
    async fn run(&self, context: &Context, interaction: &CommandInteraction) -> CommandResult {
        interaction.defer(&context.http).await?;

        let now = Utc::now();

        let options = interaction.data.options();
        let extractor = CommandOptionExtractor::new(&options);
        let character_id_raw = extractor.get_integer("character-id")?;
        let character_id = ai_chat_character::Id(character_id_raw as u32);
        let message = extractor.get_string("message")?;

        let user_id = interaction.user.id.to_string();
        let user_name = interaction.member.as_ref()
            .and_then(|member| member.nick.clone())
            .or_else(|| interaction.user.global_name.clone())
            .unwrap_or(self.fluent_proxy.get_message("ai-chat--user-name--default", None).to_string());

        let use_case_result = match self.chat_ai_use_case.run(&user_id, &user_name, character_id, message, &now).await {
            Ok(result) => result,
            Err(use_case::chat_ai_use_case::UseCaseError::ChatCountExceeded) => {
                let error_message = self.fluent_proxy.get_message("ai-chat--error--execution-limit-exceeded", None);
                interaction.create_followup(&context.http, CreateInteractionResponseFollowup::new().content(error_message)).await?;
                return Ok(());
            }
            Err(use_case::chat_ai_use_case::UseCaseError::CharacterNotFound) => {
                let error_message = self.fluent_proxy.get_message("ai-chat--error--character-not-found", None);
                interaction.create_followup(&context.http, CreateInteractionResponseFollowup::new().content(error_message)).await?;
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };

        let reference_items_message = match use_case_result.ai_text.web_references() {
            None => self.fluent_proxy.get_message("ai-chat--response--no-reference", None).to_string(),
            Some(references) => {
                references.iter().map(|reference| {
                    let fluent_args = fluent_args![
                        "title" => reference.title(),
                        "url" => reference.url(),
                    ];
                    self.fluent_proxy.get_message("ai-chat--response--reference-item", Some(&fluent_args))
                }).collect::<Vec<_>>().join("\n")
            }
        };

        let fluent_args = fluent_args![
            "user-name" => user_name,
            "user-message" => message,
            "ai-character-name" => use_case_result.ai_character_name,
            "ai-message" => use_case_result.ai_text.text(),
            "reference-items" => reference_items_message,
        ];

        let body = self.fluent_proxy.get_message("ai-chat--response--body", Some(&fluent_args));

        let is_message_length_exceeded = body.chars().count() > MAX_MESSAGE_LENGTH;
        let result_message = if is_message_length_exceeded {
            self.fluent_proxy.get_message("ai-chat--response--message-length-exceeded", None).to_string()
        } else {
            body.to_string()
        };

        let mut followup_response = CreateInteractionResponseFollowup::new().content(result_message);
        if is_message_length_exceeded {
            let attachment = CreateAttachment::bytes(
                body.as_bytes(),
                "response.md",
            );
            followup_response = followup_response.add_file(attachment);
        }

        interaction
            .create_followup(&context.http, followup_response)
            .await?;
        Ok(())
    }
}

pub struct Factory {
    ai_chat_character_repository: Arc<dyn AIChatCharacterRepository + Send + Sync>,
    chat_ai_use_case: Arc<ChatAIUseCase>,
    fluent_proxy: Arc<FluentProxy>,
}

impl Factory {
    pub fn new(
        ai_chat_character_repository: Arc<dyn AIChatCharacterRepository + Send + Sync>,
        chat_ai_use_case: Arc<ChatAIUseCase>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            ai_chat_character_repository,
            chat_ai_use_case,
            fluent_proxy,
        }
    }
}

impl CommandFactory for Factory {
    fn command_name(&self) -> String {
        "ai-chat".to_string()
    }

    fn command_specification(&self) -> CreateCommand {
        let mut character_type_option =
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "character-id",
                self.fluent_proxy.get_message("ai-chat--command-option--character-id--description", None),
            )
                .required(true);

        for ai_chat_character in self.ai_chat_character_repository.get_all() {
            character_type_option = character_type_option
                .add_int_choice(ai_chat_character.display_name(), ai_chat_character.id().0.try_into().unwrap()) // TODO: Handle error
        }

        let message_option =
            CreateCommandOption::new(
                CommandOptionType::String,
                "message",
                self.fluent_proxy.get_message("ai-chat--command-option--message--description", None),
            )
                .required(true)
                .min_length(1)
                .max_length(400);

        // TODO: Describe execution limit
        CreateCommand::new(self.command_name())
            .description(self.fluent_proxy.get_message("ai-chat--command--description", None))
            .add_option(character_type_option)
            .add_option(message_option)
    }

    fn create(&self) -> Box<dyn CommandRunner + Send + Sync> {
        Box::new(AIChat::new(
            self.chat_ai_use_case.clone(),
            self.fluent_proxy.clone(),
        ))
    }
}
