use async_trait::async_trait;
use serenity::all::{
    CommandInteraction, Context, CreateInteractionResponseFollowup, Message, ModalInteraction,
};

#[async_trait]
pub trait Followupable {
    async fn send_followup(
        &self,
        context: &Context,
        followup: CreateInteractionResponseFollowup,
    ) -> serenity::Result<Message>;
}

#[async_trait]
impl Followupable for CommandInteraction {
    async fn send_followup(
        &self,
        context: &Context,
        followup: CreateInteractionResponseFollowup,
    ) -> serenity::Result<Message> {
        self.create_followup(&context.http, followup).await
    }
}

#[async_trait]
impl Followupable for ModalInteraction {
    async fn send_followup(
        &self,
        context: &Context,
        followup: CreateInteractionResponseFollowup,
    ) -> serenity::Result<Message> {
        self.create_followup(&context.http, followup).await
    }
}
