use serde::Serialize;
use poise::serenity_prelude as serenity;
use crate::data::Context;
use crate::error::BotError;

pub mod music;

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum V2Component {
    Container(Container),
    Section(Section),
    TextDisplay(TextDisplay),
    MediaGallery(MediaGallery),
    Thumbnail(Thumbnail),
    ActionRow(ActionRow),
    Separator(Separator),
}

#[derive(Debug, Clone, Serialize)]
pub struct Container {
    #[serde(rename = "type")]
    pub kind: u8,
    pub components: Vec<V2Component>,
}

impl Container {
    pub fn new(components: Vec<V2Component>) -> Self {
        Self { kind: 17, components }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Section {
    #[serde(rename = "type")]
    pub kind: u8,
    pub components: Vec<V2Component>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessory: Option<Box<V2Component>>,
}

impl Section {
    pub fn new(components: Vec<V2Component>) -> Self {
        Self { kind: 9, components, accessory: None }
    }

    pub fn with_accessory(mut self, accessory: V2Component) -> Self {
        self.accessory = Some(Box::new(accessory));
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TextDisplay {
    #[serde(rename = "type")]
    pub kind: u8,
    pub content: String,
}

impl TextDisplay {
    pub fn new(content: String) -> Self {
        Self { kind: 10, content }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaGallery {
    #[serde(rename = "type")]
    pub kind: u8,
    pub items: Vec<MediaItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaItem {
    #[serde(rename = "type")]
    pub kind: u8,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaObject {
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Thumbnail {
    #[serde(rename = "type")]
    pub kind: u8,
    pub media: MediaObject,
}

impl Thumbnail {
    pub fn new(url: String) -> Self {
        Self { kind: 11, media: MediaObject { url } }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionRow {
    #[serde(rename = "type")]
    pub kind: u8,
    pub components: Vec<serenity::CreateButton>,
}

impl ActionRow {
    pub fn new(buttons: Vec<serenity::CreateButton>) -> Self {
        Self { kind: 1, components: buttons }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Separator {
    #[serde(rename = "type")]
    pub kind: u8,
}

impl Separator {
    pub fn new() -> Self {
        Self { kind: 14 }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AllowedMentions {
    pub parse: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct V2MessagePayload {
    pub flags: u32,
    #[serde(skip_serializing)]
    pub content: Option<String>,
    #[serde(skip_serializing)]
    pub embeds: Option<Vec<serenity::CreateEmbed>>,
    pub components: Vec<V2Component>,
    pub allowed_mentions: AllowedMentions,
}

impl V2MessagePayload {
    pub fn new(components: Vec<V2Component>) -> Self {
        Self {
            flags: 32768,
            content: None,
            embeds: None,
            components,
            allowed_mentions: AllowedMentions { parse: vec![] },
        }
    }

    pub async fn send(
        &self,
        http: &serenity::Http,
        channel_id: serenity::ChannelId,
    ) -> Result<serenity::Message, BotError> {
        let route = serenity::Route::ChannelMessages { 
            channel_id: Ord::max(
                std::num::NonZeroU64::new(1).unwrap(), 
                std::num::NonZeroU64::new(channel_id.get()).unwrap_or(std::num::NonZeroU64::new(1).unwrap())
            ).into()
        };
        
        let request = serenity::Request::new(
            route,
            serenity::http::LightMethod::Post,
        ).body(Some(serde_json::to_vec(&self).map_err(|e| BotError::Other(e.to_string()))?));
        
        let response = http.request(request).await.map_err(|e| {
            tracing::error!("[V2] Request failed: {:?}", e);
            BotError::Other(e.to_string())
        })?;
        
        let text = response.text().await.map_err(|e| BotError::Other(e.to_string()))?;
        if text.is_empty() {
             return Err(BotError::Other("Empty response from Discord".into()));
        }
        if text.contains("code") && text.contains("message") {
            tracing::error!("[V2] Discord error response: {}", text);
        }
        
        let message: serenity::Message = serde_json::from_str(&text).map_err(|e| {
            tracing::error!("[V2] Failed to deserialize message: {}. Response was: {}", e, text);
            BotError::Other(e.to_string())
        })?;
        Ok(message)
    }

    pub async fn send_interaction(&self, ctx: Context<'_>) -> Result<(), BotError> {
        let interaction = match ctx {
            poise::Context::Application(app_ctx) => Some(app_ctx.interaction),
            _ => None,
        };

        if let Some(serenity::CommandInteraction {
            token,
            ..
        }) = interaction {
            let display_content = self.content.as_deref().unwrap_or("");
            let final_content = if display_content.trim().is_empty() {
                "\u{200E}" 
            } else {
                display_content
            };
            
            let mut reply = poise::CreateReply::default().content(final_content);
            if let Some(embeds) = &self.embeds {
                for embed in embeds {
                    reply = reply.embed(embed.clone());
                }
            }

            let initial_response = match ctx.send(reply).await {
                Ok(resp) => resp,
                Err(e) => {
                    tracing::warn!("[V2] Error sending initial response: {:?}", e);
                    return Ok(());
                }
            };
            let message_id = match initial_response.message().await {
                Ok(msg) => msg.id,
                Err(e) => {
                    tracing::error!("[V2] Failed to get message from response: {:?}", e);
                    return Ok(());
                }
            };

            if let poise::Context::Application(app_ctx) = ctx {
                let route = serenity::Route::WebhookMessage {
                    webhook_id: app_ctx.interaction.application_id.get().into(),
                    token: &app_ctx.interaction.token,
                    message_id: message_id.get().into(),
                };

                let mut map = serde_json::Map::new();
                map.insert("flags".to_string(), serde_json::Value::Number(self.flags.into()));
                map.insert("components".to_string(), serde_json::to_value(&self.components).map_err(|e| BotError::Other(e.to_string()))?);
                map.insert("content".to_string(), serde_json::Value::Null);
                map.insert("embeds".to_string(), serde_json::Value::Array(vec![]));

                let body = serde_json::Value::Object(map);

                let request = serenity::Request::new(
                    route,
                    serenity::http::LightMethod::Patch,
                ).body(Some(serde_json::to_vec(&body).map_err(|e| BotError::Other(e.to_string()))?));

                // Send the patch to add buttons/containers
                if let Err(e) = app_ctx.serenity_context.http.request(request).await {
                    tracing::error!("[V2] Failed to patch interaction: {:?}", e);
                }
            }
        } else {
            self.send(&ctx.serenity_context().http, ctx.channel_id()).await?;
        }
        Ok(())
    }

    pub async fn edit(
        &self,
        http: &serenity::Http,
        channel_id: serenity::ChannelId,
        message_id: serenity::MessageId,
    ) -> Result<serenity::Message, BotError> {
        let route = serenity::Route::ChannelMessage { 
            channel_id: channel_id.get().into(),
            message_id: message_id.get().into(),
        };
        
        let request = serenity::Request::new(
            route,
            serenity::http::LightMethod::Patch,
        ).body(Some(serde_json::to_vec(&self).map_err(|e| BotError::Other(e.to_string()))?));
        
        let response = http.request(request).await.map_err(|e| {
            tracing::error!("[V2] Edit failed: {:?}", e);
            BotError::Other(e.to_string())
        })?;
        
        let text = response.text().await.map_err(|e| BotError::Other(e.to_string()))?;
        let message: serenity::Message = serde_json::from_str(&text).map_err(|e| {
            tracing::error!("[V2] Failed to deserialize edit message: {}. Response was: {}", e, text);
            BotError::Other(e.to_string())
        })?;
        Ok(message)
    }
}

pub fn createErrorV2() -> Vec<V2Component> {
    createEmbedV2(
        "An error occurred while running command", 
        "-# An unexpected error occurred. Please contact the developer for assistance.", 
        None
    )
}

pub fn createEmbedV2(
    title: &str,
    description: &str,
    _artwork_url: Option<&str>
) -> Vec<V2Component> {
    let components = vec![
        V2Component::TextDisplay(TextDisplay::new(format!("**{}**", title))),
        V2Component::TextDisplay(TextDisplay::new(description.to_string())),
    ];

    vec![
        V2Component::Container(Container::new(components))
    ]
}
