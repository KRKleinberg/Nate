use crate::{Context, Error};

/// Displays information about how to use the app
#[poise::command(
    prefix_command,
    slash_command,
    track_edits,
    aliases("h"),
    category = "Utility"
)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            ..Default::default()
        },
    )
    .await?;
    return Ok(());
}
