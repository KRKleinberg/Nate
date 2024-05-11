use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Displays the age of a user's Discord account
#[poise::command(slash_command, prefix_command, track_edits, category = "Utility")]
pub async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let user = user.as_ref().unwrap_or_else(|| ctx.author());

    if user.bot {
        let response = format!("Cannot display an app's age.");

        ctx.say(response).await?;
        return Ok(());
    }

    let response = format!(
        "{}'s account was created on {}.",
        user.global_name.as_ref().unwrap(),
        user.created_at().format("%m/%d/%Y at %H:%M")
    );

    ctx.say(response).await?;
    return Ok(());
}
