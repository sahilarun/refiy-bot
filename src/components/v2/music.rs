use super::{V2Component, Container, Section, TextDisplay, Thumbnail, ActionRow, Separator};
use poise::serenity_prelude as serenity;

pub fn createNowPlayingV2(
    title: &str,
    author: &str,
    uri: &str,
    duration: u64,
    artworkUrl: Option<&str>,
    is_paused: bool,
    requester_id: u64,
) -> Vec<V2Component> {
    let _status_emoji = if is_paused { "⏸" } else { "🎶" };
    let status_text = if is_paused { "Paused" } else { "Now Playing" };
    
    vec![
        V2Component::Container(Container::new(vec![
            V2Component::Section(
                Section::new(vec![
                    V2Component::TextDisplay(TextDisplay::new(format!("**{}**", status_text))),
                    V2Component::TextDisplay(TextDisplay::new(format!("**[{}]({})** - {}", title, uri, author))),
                    V2Component::TextDisplay(TextDisplay::new(format!("Duration: `{}`", crate::utils::formatDuration(duration)))),
                ])
                .with_accessory(V2Component::Thumbnail(Thumbnail::new(artworkUrl.unwrap_or_default().to_string())))
            ),
            
            V2Component::TextDisplay(TextDisplay::new(format!("-# Requested by <@{}>", requester_id))),
            V2Component::Separator(Separator::new()),
            V2Component::ActionRow(ActionRow::new(vec![
                    serenity::CreateButton::new("music_previous").emoji('⏮').style(serenity::ButtonStyle::Secondary),
                    serenity::CreateButton::new("music_pause")
                        .emoji(if is_paused { '▶' } else { '⏸' })
                        .style(serenity::ButtonStyle::Secondary),
                    serenity::CreateButton::new("music_skip").emoji('⏭').style(serenity::ButtonStyle::Secondary),
                    serenity::CreateButton::new("music_shuffle").emoji('🔀').style(serenity::ButtonStyle::Success),
                    serenity::CreateButton::new("music_stop").emoji('⏹').style(serenity::ButtonStyle::Danger),
                ],
            )),
        ]))
    ]
}
