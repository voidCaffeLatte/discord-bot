use crate::presentation::command::{CommandFactory, CommandResult, CommandRunner};
use common::fluent_proxy::FluentProxy;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateActionRow, CreateCommand,
    CreateCommandOption, CreateInputText, CreateInteractionResponse, CreateModal, InputTextStyle,
};
use std::sync::Arc;
use async_trait::async_trait;
use application::repository::ai_chat_character_repository::AIChatCharacterRepository;
use crate::presentation::common::serenity_extension::ResolvedValueExtension;

pub struct AIChat {
    fluent_proxy: Arc<FluentProxy>,
}

impl AIChat {
    pub fn new(
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            fluent_proxy
        }
    }
}

#[async_trait]
impl CommandRunner for AIChat {
    async fn run(&self, context: &Context, interaction: &CommandInteraction) -> CommandResult {
        let options = &interaction.data.options();
        let character_id: u32 = options
            .iter().find(|option| option.name == "character-id").unwrap()
            .value.as_integer().unwrap()
            .try_into().unwrap();

        let message_component =
            CreateInputText::new(
                InputTextStyle::Paragraph,
                self.fluent_proxy.get_message("ai-chat--modal-component--message--label", None),
                "message",
            )
                .max_length(400)
                .required(true);
        let message_component = CreateActionRow::InputText(message_component);

        let character_name_component =
            CreateInputText::new(
                InputTextStyle::Short,
                self.fluent_proxy.get_message("ai-chat--modal-component--character-id--label", None),
                "character-id")
                .max_length(10)
                .value(character_id.to_string())
                .required(true);
        let character_name_component = CreateActionRow::InputText(character_name_component);

        let modal = CreateModal::new(
            "ai-chat",
            self.fluent_proxy.get_message("ai-chat--modal--title", None),
        )
            .components(vec![message_component, character_name_component]);
        interaction
            .create_response(
                context.http.as_ref(),
                CreateInteractionResponse::Modal(modal),
            )
            .await?;

        Ok(())
    }
}

pub struct Factory {
    ai_chat_character_repository: Arc<dyn AIChatCharacterRepository + Send + Sync>,
    fluent_proxy: Arc<FluentProxy>,
}

impl Factory {
    pub fn new(
        ai_chat_character_repository: Arc<dyn AIChatCharacterRepository + Send + Sync>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            ai_chat_character_repository,
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

        // TODO: Describe execution limit
        CreateCommand::new(self.command_name())
            .description(self.fluent_proxy.get_message("ai-chat--command--description", None))
            .add_option(character_type_option)
    }

    fn create(&self) -> Box<dyn CommandRunner + Send + Sync> {
        Box::new(AIChat::new(self.fluent_proxy.clone()))
    }
}
