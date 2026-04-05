use poise::serenity_prelude as serenity;
use crate::data::Context;
use crate::error::BotError;
use futures::StreamExt;
use crate::components::v2::{V2Component, V2MessagePayload};

pub async fn paginate(
    ctx: Context<'_>,
    pages: Vec<serenity::CreateEmbed>,
) -> Result<(), BotError> {
    if pages.is_empty() {
        return Ok(());
    }

    let ctx_id = ctx.id();
    let mut current_page = 0;

    let reply = poise::CreateReply::default()
        .embed(pages[current_page].clone())
        .components(vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(format!("{}prev", ctx_id))
                .emoji('⬅')
                .style(serenity::ButtonStyle::Secondary),
            serenity::CreateButton::new(format!("{}next", ctx_id))
                .emoji('➡')
                .style(serenity::ButtonStyle::Secondary),
        ])]);

    ctx.send(reply).await?;

    let mut collector = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .timeout(std::time::Duration::from_secs(60))
        .filter(move |mci| mci.data.custom_id.starts_with(&ctx_id.to_string()))
        .stream();

    while let Some(mci) = collector.next().await {
        if mci.data.custom_id == format!("{}prev", ctx_id) {
            current_page = if current_page == 0 {
                pages.len() - 1
            } else {
                current_page - 1
            };
        } else if mci.data.custom_id == format!("{}next", ctx_id) {
            current_page = (current_page + 1) % pages.len();
        }

        mci.create_response(
            ctx.serenity_context(),
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .embed(pages[current_page].clone())
            ),
        )
        .await?;
    }

    Ok(())
}

pub async fn paginate_v2(
    ctx: Context<'_>,
    pages: Vec<Vec<V2Component>>,
) -> Result<(), BotError> {
    use crate::components::v2::ActionRow;
    
    if pages.is_empty() {
        return Ok(());
    }

    let ctx_id = ctx.id();
    let mut current_page = 0;

    let mut initial_components = pages[current_page].clone();
    initial_components.push(V2Component::ActionRow(ActionRow::new(vec![
        serenity::CreateButton::new(format!("{}prev", ctx_id))
            .emoji('⬅')
            .style(serenity::ButtonStyle::Secondary),
        serenity::CreateButton::new(format!("{}next", ctx_id))
            .emoji('➡')
            .style(serenity::ButtonStyle::Secondary),
    ])));

    let mut payload = V2MessagePayload::new(initial_components);
    let mut message = payload.send(&ctx.serenity_context().http, ctx.channel_id()).await?;

    let mut collector = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
        .author_id(ctx.author().id)
        .channel_id(ctx.channel_id())
        .message_id(message.id)
        .timeout(std::time::Duration::from_secs(60))
        .filter(move |mci| mci.data.custom_id.starts_with(&ctx_id.to_string()))
        .stream();

    while let Some(mci) = collector.next().await {
        if mci.data.custom_id == format!("{}prev", ctx_id) {
            current_page = if current_page == 0 {
                pages.len() - 1
            } else {
                current_page - 1
            };
        } else if mci.data.custom_id == format!("{}next", ctx_id) {
            current_page = (current_page + 1) % pages.len();
        }

        let mut updated_components = pages[current_page].clone();
        updated_components.push(V2Component::ActionRow(ActionRow::new(vec![
            serenity::CreateButton::new(format!("{}prev", ctx_id))
                .emoji('⬅')
                .style(serenity::ButtonStyle::Secondary),
            serenity::CreateButton::new(format!("{}next", ctx_id))
                .emoji('➡')
                .style(serenity::ButtonStyle::Secondary),
        ])));

        payload = V2MessagePayload::new(updated_components);
        message = payload.edit(&ctx.serenity_context().http, ctx.channel_id(), message.id).await?;

        mci.create_response(
            ctx.serenity_context(),
            serenity::CreateInteractionResponse::Acknowledge,
        )
        .await?;
    }

    Ok(())
}
