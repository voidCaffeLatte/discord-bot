use crate::presentation::command::{CommandFactory, CommandResult, CommandRunner};
use crate::presentation::common::command_option_extractor::CommandOptionExtractor;
use application::repository::ai_chat_character_repository::AIChatCharacterRepository;
use application::use_case::chat_ai_characters_use_case;
use application::use_case::chat_ai_characters_use_case::ChatAICharactersUseCase;
use async_trait::async_trait;
use chrono::Utc;
use common::fluent_proxy::FluentProxy;
use common::unique_collection::UniqueCollection;
use domain::model::ai_chat_character;
use fluent::fluent_args;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateAttachment, CreateCommand,
    CreateCommandOption, CreateInteractionResponseFollowup,
};
use std::sync::Arc;

pub struct AIConversation {
    chat_ai_characters_use_case: Arc<ChatAICharactersUseCase>,
    fluent_proxy: Arc<FluentProxy>,
}

impl AIConversation {
    const MAX_MESSAGE_LENGTH: u32 = 1950;
    const MAX_AI_CHAT_CHARACTER_COUNT: u32 = 5;

    pub fn new(
        chat_ai_characters_use_case: Arc<ChatAICharactersUseCase>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            chat_ai_characters_use_case,
            fluent_proxy,
        }
    }
}

#[async_trait]
impl CommandRunner for AIConversation {
    async fn run(&self, context: &Context, interaction: &CommandInteraction) -> CommandResult {
        interaction.defer(&context.http).await?;

        let now = Utc::now();

        let user_id = interaction.user.id.to_string();

        let options = interaction.data.options();
        let command_option_extractor = CommandOptionExtractor::new(&options);

        let character_ids = (1..=Self::MAX_AI_CHAT_CHARACTER_COUNT)
            .map(|index| format!("character-id-{}", index))
            .filter_map(|key| command_option_extractor.get_integer(&key).ok())
            .map(|id| ai_chat_character::Id(id as u32))
            .collect();
        let character_ids = UniqueCollection::new(character_ids);

        let theme = command_option_extractor.get_string("theme")?;

        let use_case_result = self.chat_ai_characters_use_case
            .run(&user_id, &character_ids, theme, &now).await;
        let use_case_result = match use_case_result {
            Ok(result) => result,
            Err(chat_ai_characters_use_case::Error::ChatCountExceeded) => {
                let error_message = self.fluent_proxy.get_message("ai-conversation--error--execution-limit-exceeded", None);
                interaction.create_followup(&context.http, CreateInteractionResponseFollowup::new().content(error_message)).await?;
                return Ok(());
            }
            Err(chat_ai_characters_use_case::Error::CharacterCountNotEnough) => {
                let error_message = self.fluent_proxy.get_message("i-conversation--error--character-count-not-enough", None);
                interaction.create_followup(&context.http, CreateInteractionResponseFollowup::new().content(error_message)).await?;
                return Ok(());
            }
            Err(error) => return Err(error.into())
        };

        let reference_items_message = match use_case_result.ai_text.web_references() {
            None => self.fluent_proxy.get_message("ai-conversation--response--no-reference", None).to_string(),
            Some(references) => {
                references.iter().map(|reference| {
                    let fluent_args = fluent_args![
                        "title" => reference.title(),
                        "url" => reference.url(),
                    ];
                    self.fluent_proxy.get_message("ai-conversation--response--reference-item", Some(&fluent_args))
                }).collect::<Vec<_>>().join("\n")
            }
        };

        let fluent_args = fluent_args![
            "theme" => theme,
            "ai-conversation" => use_case_result.ai_text.text(),
            "reference-items" => reference_items_message,
        ];

        let response_body = self.fluent_proxy.get_message(
            "ai-conversation--response--body", Some(&fluent_args),
        );

        let is_message_length_exceeded = response_body.chars().count() > Self::MAX_MESSAGE_LENGTH as usize;
        let response_message = if is_message_length_exceeded {
            self.fluent_proxy.get_message("ai-conversation--response--message-length-exceeded", None).to_string()
        } else {
            response_body.to_string()
        };

        let mut followup_response = CreateInteractionResponseFollowup::new().content(response_message);
        if is_message_length_exceeded {
            let attachment = CreateAttachment::bytes(
                response_body.as_bytes(),
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
    chat_ai_characters_use_case: Arc<ChatAICharactersUseCase>,
    fluent_proxy: Arc<FluentProxy>,
}

impl Factory {
    pub fn new(
        ai_chat_character_repository: Arc<dyn AIChatCharacterRepository + Send + Sync>,
        chat_ai_characters_use_case: Arc<ChatAICharactersUseCase>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            ai_chat_character_repository,
            chat_ai_characters_use_case,
            fluent_proxy,
        }
    }
}

impl CommandFactory for Factory {
    fn command_name(&self) -> String {
        "ai-conversation".to_string()
    }

    fn command_specification(&self) -> CreateCommand {
        let mut command_options: Vec<CreateCommandOption> = (1..=AIConversation::MAX_AI_CHAT_CHARACTER_COUNT)
            .map(|index| {
                let fluent_args = fluent_args![
                    "index" => index,
                ];

                let mut character_type_option = CreateCommandOption::new(
                    CommandOptionType::Integer,
                    format!("character-id-{}", index),
                    self.fluent_proxy.get_message("ai-conversation--command-option--character-id--description", Some(&fluent_args)),
                )
                    .required(index <= 2);

                for ai_chat_character in self.ai_chat_character_repository.get_all() {
                    character_type_option = character_type_option
                        .add_int_choice(ai_chat_character.display_name(), ai_chat_character.id().0.try_into().unwrap()); // TODO: Handle error
                }

                character_type_option
            }).collect();

        command_options.insert(
            0,
            CreateCommandOption::new(
                CommandOptionType::String,
                "theme",
                self.fluent_proxy.get_message("ai-conversation--command-option--theme--description", None),
            )
                .required(true)
                .min_length(1)
                .max_length(150),
        );

        // TODO: Describe execution limit
        CreateCommand::new(self.command_name())
            .description(self.fluent_proxy.get_message("ai-conversation--command--description", None))
            .set_options(command_options)
    }

    fn create(&self) -> Box<dyn CommandRunner + Send + Sync> {
        Box::new(AIConversation::new(
            self.chat_ai_characters_use_case.clone(),
            self.fluent_proxy.clone(),
        ))
    }
}
