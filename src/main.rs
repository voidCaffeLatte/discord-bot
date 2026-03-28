use serde_json::json;
use serenity::all::{Command, Context, CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, EventHandler, GatewayIntents, Interaction, Ready};
use serenity::{async_trait, Client};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use dashmap::DashSet;
use fluent::FluentResource;
use fluent::concurrent::FluentBundle;
use tracing::{debug, error, info};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::util::SubscriberInitExt;
use unic_langid::LanguageIdentifier;
use application::use_case::chat_ai_characters_use_case::ChatAICharactersUseCase;
use application::use_case::chat_ai_use_case::ChatAIUseCase;
use application::use_case::generate_ai_image_use_case::GenerateAIImageUseCase;
use application::use_case::get_random_twitch_clip_use_case::GetRandomTwitchClipUseCase;
use crate::presentation::{command, modal_interaction};
use crate::presentation::command::CommandFactory;
use crate::presentation::modal_interaction::ModalInteractionFactory;
use application::use_case::get_random_you_tube_video_use_case::GetRandomYouTubeVideoUseCase;
use gemini::GeminiClient;
use infrastructure::gateway::gemini_text_generation_gateway::GeminiTextGenerationGateway;
use infrastructure::gateway::gemini_image_generation_gateway::GeminiImageGenerationGateway;
use common::fluent_proxy::FluentProxy;
use infrastructure::gateway::twitch::access_token_gateway::AccessTokenGateway;
use infrastructure::gateway::you_tube;
use infrastructure::repository::ai_chat;
use infrastructure::repository::in_memory_ai_chat_history_repository::InMemoryAIChatHistoryRepository;
use infrastructure::repository::in_memory_ai_chat_character_repository::InMemoryAIChatCharacterRepository;
use infrastructure::repository::in_memory_ai_chat_activity_repository::InMemoryAIChatActivityRepository;
use infrastructure::repository::in_memory_image_generation_activity_repository::InMemoryImageGenerationActivityRepository;

mod presentation;

#[tokio::main]
async fn main() {
    // Setup i18n system
    let lang: LanguageIdentifier = "ja-JP".parse().unwrap();
    let mut bundle = FluentBundle::new_concurrent(vec![lang]);
    let source = std::fs::read_to_string("./resource/locale/ja-JP.ftl")
        .expect("File \"./resource/locale/ja-JP.ftl\" exists and can be read");
    let resource = FluentResource::try_new(source)
        .expect("Can create Fluent resource from File \"./resource/locale/ja-JP.ftl\"");
    bundle.add_resource(resource).unwrap();
    let source = std::fs::read_to_string("./resource/locale/ja-JP.override.ftl");
    if let Ok(source) = source {
        let resource = FluentResource::try_new(source)
            .expect("Can create Fluent resource from File \"./resource/locale/ja-JP.override.ftl\"");
        bundle.add_resource_overriding(resource);
        bundle.set_use_isolating(false); // Japanese-only, no bidirectional text needed
    }
    let fluent_proxy = Arc::new(FluentProxy::new(bundle));

    // Setup log system
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env().add_directive(LevelFilter::INFO.into()))
        .finish();
    subscriber.init();

    // Setup dependencies

    let http_client = reqwest::Client::new();

    let gemini_api_key = std::env::var("GEMINI_API_KEY").expect("Environment variable \"GEMINI_API_KEY\" is specified");
    let gemini_text_client = GeminiClient::new(
        gemini_api_key.clone(), reqwest::Client::new(), "gemini-3-flash-preview".to_string(),
    );
    let ai_text_generation_gateway = Arc::new(GeminiTextGenerationGateway::new(gemini_text_client));

    let twitch_app_client_id = std::env::var("TWITCH_APP_CLIENT_ID").expect("Environment variable \"TWITCH_APP_CLIENT_ID\" is specified");
    let twitch_app_client_secret = std::env::var("TWITCH_APP_CLIENT_SECRET").expect("Environment variable \"TWITCH_APP_CLIENT_SECRET\" is specified");
    let twitch_access_token_gateway = Arc::new(AccessTokenGateway::new(
        http_client.clone(),
        twitch_app_client_id.clone(),
        twitch_app_client_secret.clone(),
    ));
    let twitch_user_gateway = Arc::new(infrastructure::gateway::twitch::user_gateway::UserGateway::new(
        http_client.clone(),
        twitch_app_client_id.clone(),
        twitch_access_token_gateway.clone(),
    ));
    let twitch_clip_gateway = Arc::new(infrastructure::gateway::twitch::clip_gateway::ClipGateway::new(
        http_client.clone(),
        twitch_app_client_id.clone(),
        twitch_access_token_gateway.clone(),
    ));
    let get_random_twitch_clip_use_case = Arc::new(GetRandomTwitchClipUseCase::new(
        twitch_clip_gateway.clone(),
        twitch_user_gateway.clone(),
    ));

    let ai_chat_character_repository = Arc::new(
        InMemoryAIChatCharacterRepository::try_new(Path::new("./resource/character.toml"))
            .expect("Valid file \"resource/character.toml\" is required")
    );
    let ai_chat_history_repository = Arc::new(InMemoryAIChatHistoryRepository::new());
    let ai_chat_activity_repository = Arc::new(InMemoryAIChatActivityRepository::new());
    let ai_chat_character_relationship_repository = Arc::new(ai_chat::character::in_memory_relationship_repository::InMemoryRelationshipRepository::new());
    let chat_ai_use_case = Arc::new(ChatAIUseCase::new(
        ai_text_generation_gateway.clone(),
        ai_chat_character_repository.clone(),
        ai_chat_history_repository.clone(),
        ai_chat_activity_repository.clone(),
        ai_chat_character_relationship_repository.clone(),
        fluent_proxy.clone(),
    ));
    let chat_ai_characters_use_case = Arc::new(
        ChatAICharactersUseCase::new(
            ai_text_generation_gateway.clone(),
            ai_chat_character_repository.clone(),
            ai_chat_activity_repository.clone(),
            fluent_proxy.clone(),
        ));

    let gemini_image_client = GeminiClient::new(
        gemini_api_key, reqwest::Client::new(), "gemini-3.1-flash-image-preview".to_string(),
    );
    let gemini_image_generation_gateway = Arc::new(GeminiImageGenerationGateway::new(gemini_image_client));
    let image_generation_activity_repository = Arc::new(InMemoryImageGenerationActivityRepository::new());
    let generate_ai_image_use_case = Arc::new(GenerateAIImageUseCase::new(
        gemini_image_generation_gateway.clone(),
        image_generation_activity_repository.clone(),
    ));

    let you_tube_data_api_key = std::env::var("YOU_TUBE_DATA_API_KEY").expect("Environemnt variable \"YOU_TUBE_DATA_API_KEY\" is specified");
    let you_tube_channel_gateway = Arc::new(you_tube::channel_gateway::ChannelGateway::new(
        you_tube_data_api_key.clone(),
        http_client.clone(),
    ));
    let you_tube_playlist_item_gateway = Arc::new(you_tube::playlist_item_gateway::PlaylistItemGateway::new(
        you_tube_data_api_key.clone(),
        http_client.clone(),
    ));
    let you_tube_video_gateway = Arc::new(you_tube::video_gateway::VideoGateway::new(
        you_tube_data_api_key.clone(),
        http_client.clone(),
    ));
    let get_random_you_tube_video_use_case = Arc::new(GetRandomYouTubeVideoUseCase::new(
        you_tube_channel_gateway.clone(),
        you_tube_playlist_item_gateway.clone(),
        you_tube_video_gateway.clone(),
    ));

    let token = std::env::var("DISCORD_TOKEN").unwrap();
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES;

    let command_factories: Vec<Box<dyn CommandFactory + Send + Sync>> = vec![
        Box::new(command::ping::Factory::new(
            fluent_proxy.clone()
        )),
        Box::new(command::choices::Factory::new(
            fluent_proxy.clone(),
        )),
        Box::new(command::ai_chat::Factory::new(
            ai_chat_character_repository.clone(),
            fluent_proxy.clone(),
        )),
        Box::new(command::ai_conversation::Factory::new(
            ai_chat_character_repository.clone(),
            chat_ai_characters_use_case.clone(),
            fluent_proxy.clone(),
        )),
        Box::new(command::random_you_tube_video::Factory::new(
            get_random_you_tube_video_use_case.clone(),
            fluent_proxy.clone(),
        )),
        Box::new(command::ai_image::Factory::new(
            generate_ai_image_use_case.clone(),
            fluent_proxy.clone(),
        )),
        Box::new(command::random_twitch_clip::Factory::new(
            get_random_twitch_clip_use_case.clone(),
            fluent_proxy.clone(),
        )),
    ];

    // Setup command and modal routes

    let command_factories = command_factories
        .into_iter()
        .map(|factory| (factory.command_name(), factory))
        .collect::<HashMap<_, _>>();

    let modal_interaction_factories: Vec<Box<dyn ModalInteractionFactory + Send + Sync>> =
        vec![Box::new(modal_interaction::ai_chat::Factory::new(
            chat_ai_use_case.clone(),
            fluent_proxy.clone(),
        ))];

    let modal_interaction_factories = modal_interaction_factories
        .into_iter()
        .map(|factory| (factory.modal_name(), factory))
        .collect::<HashMap<_, _>>();

    // Run discord bot

    let handler = Handler::new(command_factories, modal_interaction_factories, http_client.clone(), fluent_proxy.clone());
    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Failed to build Discord client.");

    if let Err(reason) = client.start().await {
        error!("Discord bot crashed: {}", reason);
    }
}

struct Handler {
    command_factories: HashMap<String, Box<dyn CommandFactory + Send + Sync>>,
    modal_interaction_factories: HashMap<String, Box<dyn ModalInteractionFactory + Send + Sync>>,
    running_users: DashSet<String>,
    http_client: reqwest::Client,
    fluent_proxy: Arc<FluentProxy>,
}

impl Handler {
    fn new(
        command_factories: HashMap<String, Box<dyn CommandFactory + Send + Sync>>,
        modal_interaction_factories: HashMap<String, Box<dyn ModalInteractionFactory + Send + Sync>>,
        http_client: reqwest::Client,
        fluent_proxy: Arc<FluentProxy>,
    ) -> Self {
        Self {
            command_factories,
            modal_interaction_factories,
            running_users: DashSet::new(),
            http_client,
            fluent_proxy,
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: Context, ready: Ready) {
        info!("Registering all commands...");
        let command_specifications = self
            .command_factories
            .values()
            .map(|factory| factory.command_specification())
            .collect();
        Command::set_global_commands(&context.http, command_specifications)
            .await
            .expect("Failed to update global commands.");
        info!("Finished!");

        info!("Discord bot is ready!: {}", ready.user.name);
    }

    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command_interaction) => {
                if command_interaction.user.bot { return; };

                let command_name = command_interaction.data.name.clone();
                if let Some(command_factory) = self.command_factories.get(command_name.as_str()) {
                    let user_id = command_interaction.user.id.to_string();
                    if !self.running_users.insert(user_id.clone()) {
                        let error_message = CreateInteractionResponseMessage::new().content(self.fluent_proxy.get_message("common--error--already-running-command", None));
                        command_interaction.create_response(&context.http, CreateInteractionResponse::Message(error_message)).await.unwrap(); // TODO: Handle error
                        return;
                    }

                    let command = command_factory.create();
                    if let Err(error) = command.run(&context, &command_interaction).await {
                        debug!("Unhandled Error Occurred!: {:?}", error);
                        error!("Unhandled Error Occurred!: {}", error);

                        self.send_log_to_webhook(&format!("## ERROR\n```\n{}\n{:?}\n```", error, error)).await;

                        command_interaction
                            .create_followup(
                                &context.http,
                                CreateInteractionResponseFollowup::new()
                                    .content(self.fluent_proxy.get_message("common--error--unexpected-error-occurred", None)),
                            )
                            .await
                            .unwrap(); // FIXME: Will panic if already followed up
                    }

                    // Add 1-second cooldown for every command execution
                    tokio::time::sleep(Duration::from_secs(1)).await;

                    let _ = self.running_users.remove(&user_id);
                }
            }
            Interaction::Modal(interaction) => {
                if interaction.user.bot { return; };

                let modal_name = interaction.data.custom_id.clone();
                if let Some(modal_interaction_factory) =
                    self.modal_interaction_factories.get(modal_name.as_str())
                {
                    let modal_interaction = modal_interaction_factory.create();
                    if let Err(error) = modal_interaction.run(&context, &interaction).await {
                        debug!("Unhandled Error Occurred!: {:?}", error);
                        error!("Unhandled Error Occurred!: {}", error);

                        self.send_log_to_webhook(&format!("## ERROR\n```\n{}\n{:?}\n```", error, error)).await;

                        interaction
                            .create_followup(
                                &context.http,
                                CreateInteractionResponseFollowup::new()
                                    .content(self.fluent_proxy.get_message("common--error--unexpected-error-occurred", None)),
                            )
                            .await
                            .unwrap(); // FIXME: Will panic if already followed up
                    }
                }
            }
            _ => {}
        };
    }
}

impl Handler {
    // TODO: Split into modules
    async fn send_log_to_webhook(&self, message: &str) -> ()
    {
        let Ok(url) = std::env::var("LOGGING_WEB_HOOK_URL") else {
            error!("No logging webhook URL provided.");
            return;
        };

        let json = json!({ "content": message });
        let result = self.http_client
            .post(url)
            .json(&json)
            .send()
            .await;

        if let Err(error) = result {
            error!("Unhandled Error Occurred!: {}", error);
        };
    }
}
