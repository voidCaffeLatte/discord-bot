use async_trait::async_trait;
use serenity::prelude::Context;

pub mod ai_chat;

#[async_trait]
pub trait ModalInteraction {
    async fn run(
        &self,
        context: &Context,
        interaction: &serenity::all::ModalInteraction,
    ) -> ModalInteractionResult;
}

pub trait ModalInteractionFactory {
    fn modal_name(&self) -> String;

    fn create(&self) -> Box<dyn ModalInteraction + Send + Sync>;
}

pub type ModalInteractionError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub type ModalInteractionResult = Result<(), ModalInteractionError>;
