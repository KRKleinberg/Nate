#[macro_use]
extern crate tracing;

pub mod commands {
    pub mod utility {
        pub mod age;
        pub mod help;
    }
    pub mod music {
        pub mod play;
    }
}
pub mod events {
    pub mod music;
}

use dotenv::dotenv;

use lavalink_rs::{model, prelude::*};

use poise::serenity_prelude as serenity;
use songbird::SerenityInit;

pub struct Data {
    pub lavalink: LavalinkClient,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start app: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command: `{}`: {:?}", ctx.command().name, error)
        }
        error => {
            if let Err(err) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {:?}", err)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "info,lavalink_rs=trace");
    tracing_subscriber::fmt::init();

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::utility::age::age(),
            commands::utility::help::help(),
            commands::music::play::play(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("nate".into()),
            edit_tracker: Some(std::sync::Arc::new(poise::EditTracker::for_timespan(
                std::time::Duration::from_secs(3600),
            ))),
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    };

    let token =
        std::env::var("DISCORD_BOT_TOKEN").expect("Expected DISCORD_BOT_TOKEN in the environment");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                let events = model::events::Events {
                    raw: Some(events::music::raw_event),
                    ready: Some(events::music::ready_event),
                    track_start: Some(events::music::track_start),
                    ..Default::default()
                };

                let node_local = NodeBuilder {
                    hostname: "localhost:2333".to_string(),
                    is_ssl: false,
                    events: model::events::Events::default(),
                    password: std::env::var("LAVALINK_PASSWORD")
                        .expect("Expected LAVALINK_PASSWORD in the environment"),
                    user_id: ctx.cache.current_user().id.into(),
                    session_id: None,
                };

                let client = LavalinkClient::new(
                    events,
                    vec![node_local],
                    NodeDistributionStrategy::round_robin(),
                )
                .await;

                Ok(Data { lavalink: client })
            })
        })
        .options(options)
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .register_songbird()
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(())
}
