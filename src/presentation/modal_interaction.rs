use async_trait::async_trait;
use serenity::prelude::Context;

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

pub type ModalInteractionError = anyhow::Error;

pub type ModalInteractionResult = Result<(), ModalInteractionError>;
