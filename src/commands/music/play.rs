use crate::Context;
use crate::Error;

use std::collections::VecDeque;
use std::ops::Deref;

use lavalink_rs::prelude::*;

use poise::serenity_prelude as serenity;
use serenity::{model::id::ChannelId, Http, Mentionable};

async fn _join(
    ctx: &Context<'_>,
    guild_id: serenity::GuildId,
    channel_id: Option<serenity::ChannelId>,
) -> Result<bool, Error> {
    let lava_client = ctx.data().lavalink.clone();

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();

    if lava_client.get_player_context(guild_id).is_none() {
        let connect_to = match channel_id {
            Some(x) => x,
            None => {
                let guild = ctx.guild().unwrap().deref().clone();
                let user_channel_id = guild
                    .voice_states
                    .get(&ctx.author().id)
                    .and_then(|voice_state| voice_state.channel_id);

                match user_channel_id {
                    Some(channel) => channel,
                    None => {
                        ctx.say("Not in a voice channel").await?;

                        return Err("Not in a voice channel".into());
                    }
                }
            }
        };

        let handler = manager.join_gateway(guild_id, connect_to).await;

        match handler {
            Ok((connection_info, _)) => {
                lava_client
                    // The turbofish here is Optional, but it helps to figure out what type to
                    // provide in `PlayerContext::data()`
                    //
                    // While a tuple is used here as an example, you are free to use a custom
                    // public structure with whatever data you wish.
                    // This custom data is also present in the Client if you wish to have the
                    // shared data be more global, rather than centralized to each player.
                    .create_player_context_with_data::<(ChannelId, std::sync::Arc<Http>)>(
                        guild_id,
                        connection_info,
                        std::sync::Arc::new((
                            ctx.channel_id(),
                            ctx.serenity_context().http.clone(),
                        )),
                    )
                    .await?;

                ctx.say(format!("Joined {}", connect_to.mention())).await?;

                return Ok(true);
            }
            Err(why) => {
                ctx.say(format!("Error joining the channel: {}", why))
                    .await?;
                return Err(why.into());
            }
        }
    }

    Ok(false)
}

#[poise::command(slash_command, prefix_command)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "Search term or URL"]
    #[rest]
    term: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let _ = _join(&ctx, guild_id, None).await?;
    let lava_client = ctx.data().lavalink.clone();

    let Some(player) = lava_client.get_player_context(guild_id) else {
        ctx.say("Join the bot to a voice channel first.").await?;
        return Ok(());
    };

    let query = if let Some(term) = term {
        if term.starts_with("http") {
            term
        } else {
            SearchEngines::Spotify.to_query(&term)?
        }
    } else {
        if let Ok(player_data) = player.get_player().await {
            let queue = player.get_queue().await.unwrap();

            if player_data.track.is_none() && queue.is_empty() {
                player.skip()?;
            } else {
                ctx.say("The queue is empty.").await?;
            }
        }

        return Ok(());
    };
    let loaded_tracks = lava_client.load_tracks(guild_id, &query).await?;

    let mut playlist_info = None;

    let mut tracks: VecDeque<TrackInQueue> = match loaded_tracks.data {
        Some(TrackLoadData::Track(x)) => {
            let mut v: VecDeque<TrackInQueue> = VecDeque::new();
            v.push_back(TrackInQueue::from(x));
            v
        }
        Some(TrackLoadData::Search(x)) => vec![x[0].clone().into()].into(),
        Some(TrackLoadData::Playlist(x)) => {
            println!("Playlist");
            playlist_info = Some(x.info);
            x.tracks
                .iter()
                .map(|x| x.clone().into())
                .collect::<Vec<_>>()
                .into_iter()
                .collect()
        }
        _ => {
            ctx.say(format!("{:?}", loaded_tracks)).await?;
            return Ok(());
        }
    };

    if let Some(info) = playlist_info {
        let _ = ctx
            .say(format!("Added playlist to queue: {}", info.name,))
            .await?;
    } else {
        let track = tracks[0].track.clone();

        if let Some(uri) = &track.info.uri {
            ctx.say(format!(
                "Added to queue: [{} - {}](<{}>)",
                track.info.author, track.info.title, uri
            ))
            .await?;
        } else {
            ctx.say(format!(
                "Added to queue: {} - {}",
                track.info.author, track.info.title
            ))
            .await?;
        }
    }

    let mut queue: VecDeque<TrackInQueue> = player.get_queue().await.expect("Err");
    queue.append(&mut tracks); //add tacks

    if let Ok(_player_data) = player.get_player().await {
        player.play(&queue.get(0).unwrap().track).await?;
    }

    Ok(())
}

/// Join the specified voice channel or the one you are currently in.
#[poise::command(slash_command, prefix_command)]
pub async fn join(
    ctx: Context<'_>,
    #[description = "The channel ID to join to."]
    #[channel_types("Voice")]
    channel_id: Option<serenity::ChannelId>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    _join(&ctx, guild_id, channel_id).await?;

    Ok(())
}
/// Leave the current voice channel.
#[poise::command(slash_command, prefix_command)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context()).await.unwrap().clone();
    let lava_client = ctx.data().lavalink.clone();

    lava_client.delete_player(guild_id).await?;

    if manager.get(guild_id).is_some() {
        manager.remove(guild_id).await?;
    }

    ctx.say("Left voice channel.").await?;

    Ok(())
}
