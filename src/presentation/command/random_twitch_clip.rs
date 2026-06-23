use crate::presentation::command::{CommandFactory, CommandResult, CommandRunner};
use crate::presentation::common::duration_formatter::format_duration;
use crate::presentation::common::serenity_extension::ResolvedValueExtension;
use application::use_case::get_random_twitch_clip_use_case;
use application::use_case::get_random_twitch_clip_use_case::GetRandomTwitchClipUseCase;
use async_trait::async_trait;
use chrono::Utc;
use common::fluent_proxy::FluentProxy;
use domain::value_object::twitch::user_login_id;
use domain::value_object::twitch::user_login_id::UserLoginId;
use fluent::fluent_args;
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponseFollowup,
};
use std::sync::Arc;

pub struct RandomTwitchClip {
    get_random_twitch_clip_use_case: Arc<GetRandomTwitchClipUseCase>,
    fluent_proxy: Arc<FluentProxy>,
}

impl RandomTwitchClip {
    pub fn new(
        get_random_twitch_clip_use_case: Arc<GetRandomTwitchClipUseCase>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            get_random_twitch_clip_use_case,
            fluent_proxy,
        }
    }
}

#[async_trait]
impl CommandRunner for RandomTwitchClip {
    async fn run(&self, context: &Context, interaction: &CommandInteraction) -> CommandResult {
        interaction.defer(&context.http).await?;

        let now = Utc::now();

        let options = interaction.data.options();
        let user_id = options
            .iter()
            .find(|option| option.name == "user-id")
            .unwrap()
            .value
            .as_string()
            .unwrap();
        let user_id = match UserLoginId::try_new(user_id.to_string()) {
            Ok(user_id) => user_id,
            Err(user_login_id::Error::InvalidFormat | user_login_id::Error::Empty) => {
                let error_message = self
                    .fluent_proxy
                    .get_message("random-twitch-clip--error--invalid-user-id", None);
                interaction
                    .create_followup(
                        &context.http,
                        CreateInteractionResponseFollowup::new().content(error_message),
                    )
                    .await?;
                return Ok(());
            }
        };
        let amount = options
            .iter()
            .find(|option| option.name == "amount")
            .and_then(|option| option.value.as_integer())
            .unwrap_or(1) as usize;

        let clips = match self
            .get_random_twitch_clip_use_case
            .run(&user_id, amount, &now)
            .await
        {
            Ok(result) if !result.is_empty() => result,
            Ok(_) => {
                let error_message = self
                    .fluent_proxy
                    .get_message("random-twitch-clip--error--clip-not-found", None)
                    .to_string();
                interaction
                    .create_followup(
                        &context.http,
                        CreateInteractionResponseFollowup::new().content(error_message),
                    )
                    .await?;
                return Ok(());
            }
            Err(get_random_twitch_clip_use_case::Error::UserNotFound) => {
                let error_message = self
                    .fluent_proxy
                    .get_message("random-twitch-clip--error--user-not-found", None)
                    .to_string();
                interaction
                    .create_followup(
                        &context.http,
                        CreateInteractionResponseFollowup::new().content(error_message),
                    )
                    .await?;
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };

        let clips_message = clips.iter().map(|clip| {
            let fluent_args = fluent_args![
                "title" => clip.title(),
                "duration" => format_duration(clip.duration()),
                "view-count" => clip.view_count(),
                "created-at" => clip.created_at().with_timezone(&chrono_tz::Asia::Tokyo).to_rfc3339(),
                "url" => clip.url(),
            ];
            self.fluent_proxy.get_message("random-twitch-clip--response--clip", Some(&fluent_args))
        }).collect::<Vec<_>>().join("\n");

        let fluent_args = fluent_args![
            "user-login-id" => user_id.id(),
            "clips" => clips_message,
        ];
        let result_message = self
            .fluent_proxy
            .get_message("random-twitch-clip--response--body", Some(&fluent_args));

        let followup_response = CreateInteractionResponseFollowup::new().content(result_message);
        interaction
            .create_followup(&context.http, followup_response)
            .await?;
        Ok(())
    }
}

pub struct Factory {
    get_random_twitch_clip_use_case: Arc<GetRandomTwitchClipUseCase>,
    fluent_proxy: Arc<FluentProxy>,
}

impl Factory {
    pub fn new(
        get_random_twitch_clip_use_case: Arc<GetRandomTwitchClipUseCase>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            get_random_twitch_clip_use_case,
            fluent_proxy,
        }
    }
}

impl CommandFactory for Factory {
    fn command_name(&self) -> String {
        "random-twitch-clip".to_string()
    }

    fn command_specification(&self) -> CreateCommand {
        let command_option_description = self.fluent_proxy.get_message(
            "random-twitch-clip--command-option--user-id--description",
            None,
        );
        let user_id = CreateCommandOption::new(
            CommandOptionType::String,
            "user-id",
            command_option_description,
        )
        .required(true)
        .min_length(1)
        .max_length(50);

        let command_option_description = self.fluent_proxy.get_message(
            "random-twitch-clip--command-option--amount--description",
            None,
        );
        let amount = CreateCommandOption::new(
            CommandOptionType::Integer,
            "amount",
            command_option_description,
        )
        .required(false)
        .min_int_value(1)
        .max_int_value(3);

        let command_description = self
            .fluent_proxy
            .get_message("random-twitch-clip--command--description", None);
        CreateCommand::new(self.command_name())
            .description(command_description)
            .add_option(user_id)
            .add_option(amount)
    }

    fn create(&self) -> Box<dyn CommandRunner + Send + Sync> {
        Box::new(RandomTwitchClip::new(
            self.get_random_twitch_clip_use_case.clone(),
            self.fluent_proxy.clone(),
        ))
    }
}
