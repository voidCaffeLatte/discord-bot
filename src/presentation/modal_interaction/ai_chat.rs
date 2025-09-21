use crate::presentation::common::serenity_extension::ActionRowComponentExtension;
use crate::presentation::modal_interaction::{ModalInteraction, ModalInteractionFactory, ModalInteractionResult};
use application::use_case;
use application::use_case::chat_ai_use_case::ChatAIUseCase;
use async_trait::async_trait;
use chrono::Utc;
use common::fluent_proxy::FluentProxy;
use domain::model::ai_chat_character;
use fluent::fluent_args;
use serenity::all::{Context, CreateAttachment, CreateInteractionResponseFollowup};
use std::sync::Arc;

const MAX_MESSAGE_LENGTH: u32 = 1950;

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
impl ModalInteraction for AIChat {
    async fn run(&self, context: &Context, interaction: &serenity::all::ModalInteraction) -> ModalInteractionResult {
        interaction.defer(&context.http).await?;

        let now = Utc::now();

        let user_id = interaction.user.id.to_string();
        let user_name = interaction.member.clone()
            .and_then(|member| member.nick)
            .or_else(|| interaction.user.global_name.clone())
            .unwrap_or(self.fluent_proxy.get_message("ai-chat--user-name--default", None).to_string());

        let components = interaction.data.components
            .iter().flat_map(|component| &component.components).collect::<Vec<_>>();
        let character_id = components
            .iter()
            .find(|component| component.custom_id().as_deref() == Some("character-id"))
            .and_then(|component| component.as_input_text())
            .and_then(|input_text| input_text.value.as_ref())
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| "Failed to retrieve \"character-id\" value".to_string())?;
        let character_id = ai_chat_character::Id(character_id);

        let message = components
            .iter().find(|component| component.custom_id().as_deref() == Some("message")).unwrap()
            .as_input_text().unwrap()
            .value.clone().unwrap();

        let use_case_result = match self.chat_ai_use_case.run(&user_id, &user_name, character_id, &message, &now).await {
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

        let is_message_length_exceeded = body.chars().count() > MAX_MESSAGE_LENGTH as usize;
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
    chat_ai_use_case: Arc<ChatAIUseCase>,
    fluent_proxy: Arc<FluentProxy>,
}

impl Factory {
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

impl ModalInteractionFactory for Factory {
    fn modal_name(&self) -> String {
        "ai-chat".to_string()
    }

    fn create(&self) -> Box<dyn ModalInteraction + Send + Sync> {
        Box::new(AIChat::new(self.chat_ai_use_case.clone(), self.fluent_proxy.clone()))
    }
}
