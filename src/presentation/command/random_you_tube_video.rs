use crate::presentation::command::{CommandFactory, CommandResult, CommandRunner};
use crate::presentation::common::duration_formatter::format_duration;
use crate::presentation::common::serenity_extension::ResolvedValueExtension;
use application::use_case::get_random_you_tube_video_use_case;
use application::use_case::get_random_you_tube_video_use_case::GetRandomYouTubeVideoUseCase;
use async_trait::async_trait;
use chrono::Utc;
use common::fluent_proxy::FluentProxy;
use domain::value_object::you_tube::channel_handle;
use domain::value_object::you_tube::channel_handle::ChannelHandle;
use fluent::fluent_args;
use serenity::all::{CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption, CreateInteractionResponseFollowup};
use std::sync::Arc;

pub struct RandomYouTubeVideo {
    get_random_you_tube_video_use_case: Arc<GetRandomYouTubeVideoUseCase>,
    fluent_proxy: Arc<FluentProxy>,
}

impl RandomYouTubeVideo {
    pub fn new(
        get_random_you_tube_video_use_case: Arc<GetRandomYouTubeVideoUseCase>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            get_random_you_tube_video_use_case,
            fluent_proxy,
        }
    }
}

#[async_trait]
impl CommandRunner for RandomYouTubeVideo {
    async fn run(&self, context: &Context, interaction: &CommandInteraction) -> CommandResult {
        interaction.defer(&context.http).await?;

        let now = Utc::now();

        let options = interaction.data.options();
        let handle = options.iter().find(|option| option.name == "handle").unwrap().value.as_string().unwrap();

        let handle = match ChannelHandle::try_new(handle.to_string()) {
            Ok(handle) => handle,
            Err(channel_handle::Error::InvalidLength(_min, _max, _length)) => {
                let error_message = self.fluent_proxy.get_message("random-you-tube-video--error--invalid-channel-handle", None);
                interaction.create_followup(&context.http, CreateInteractionResponseFollowup::new().content(error_message)).await?;
                return Ok(());
            }
        };

        let output = match self.get_random_you_tube_video_use_case.run(&handle, &now).await {
            Ok(video) => video,
            Err(get_random_you_tube_video_use_case::Error::ChannelNotFound) => {
                let error_message = self.fluent_proxy.get_message("random-you-tube-video--error--channel-not-found", None);
                interaction.create_followup(&context.http, CreateInteractionResponseFollowup::new().content(error_message)).await?;
                return Ok(());
            }
            Err(get_random_you_tube_video_use_case::Error::VideoNotFound) => {
                let error_message = self.fluent_proxy.get_message("random-you-tube-video--error--video-not-found", None);
                interaction.create_followup(&context.http, CreateInteractionResponseFollowup::new().content(error_message)).await?;
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        };
        let video = output.you_tube_video;

        let title = video.title();
        let duration = format_duration(video.duration());
        let view_count = video.view_count().to_string();
        let published_at = video.published_at().with_timezone(&chrono_tz::Asia::Tokyo).to_rfc3339();

        let fluent_args = fluent_args![
            "title" => title,
            "duration" => duration,
            "view-count" => view_count,
            "published-at" => published_at,
            "url" => video.video_url(),
        ];
        let video_message = self.fluent_proxy.get_message("random-you-tube-video--response--video", Some(&fluent_args));

        let fluent_args = fluent_args![
            "channel-title" => output.you_tube_channel.title(),
            "channel-handle" => handle.handle(),
            "videos" => video_message,
        ];
        let result_message = self.fluent_proxy.get_message("random-you-tube-video--response--body", Some(&fluent_args));

        let response_message = CreateInteractionResponseFollowup::new()
            .content(result_message);
        interaction.create_followup(&context.http, response_message).await?;

        Ok(())
    }
}

pub struct Factory {
    get_random_you_tube_video_use_case: Arc<GetRandomYouTubeVideoUseCase>,
    fluent_proxy: Arc<FluentProxy>,
}

impl Factory {
    pub fn new(
        get_random_you_tube_video_use_case: Arc<GetRandomYouTubeVideoUseCase>,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            get_random_you_tube_video_use_case,
            fluent_proxy,
        }
    }
}

impl CommandFactory for Factory {
    fn command_name(&self) -> String {
        "random-youtube-video".to_string()
    }

    fn command_specification(&self) -> CreateCommand {
        let handle_option = CreateCommandOption::new(
            CommandOptionType::String,
            "handle",
            self.fluent_proxy.get_message("random-you-tube-video--command-option--handle--description", None),
        )
            .required(true)
            .min_length(ChannelHandle::MIN_LENGTH as u16)
            .max_length(ChannelHandle::MAX_LENGTH as u16);

        CreateCommand::new(self.command_name())
            .description(self.fluent_proxy.get_message("random-you-tube-video--command--description", None))
            .add_option(handle_option)
    }

    fn create(&self) -> Box<dyn CommandRunner + Send + Sync> {
        Box::new(RandomYouTubeVideo::new(
            self.get_random_you_tube_video_use_case.clone(),
            self.fluent_proxy.clone(),
        ))
    }
}
