use crate::data::Context;
use crate::error::BotError;

#[poise::command(slash_command, prefix_command, category = "Information")]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Command to get help for"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), BotError> {
    let configuration = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "Type /help <command> for more info on a specific command.",
        ..Default::default()
    };

    poise::builtins::help(ctx, command.as_deref(), configuration).await?;
    Ok(())
}
