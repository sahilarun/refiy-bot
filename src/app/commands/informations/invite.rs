use poise::serenity_prelude as serenity;
use crate::data::Context;
use crate::error::BotError;

#[poise::command(slash_command, prefix_command, category = "Information")]
pub async fn invite(ctx: Context<'_>) -> Result<(), BotError> {
    let bot_id = ctx.framework().bot_id;
    let invite_url = format!(
        "https://discord.com/api/oauth2/authorize?client_id={}&permissions=8&scope=bot%20applications.commands",
        bot_id
    );

    let embed = serenity::CreateEmbed::new()
        .title("Invite Me")
        .description(format!("Click [here]({}) to invite me to your server!", invite_url))
        .color(ctx.data().config.colorPrimary)
        .thumbnail(ctx.cache().current_user().face());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
