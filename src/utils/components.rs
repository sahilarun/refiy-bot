use poise::serenity_prelude as serenity;

pub fn createMusicButtons() -> serenity::CreateActionRow {
    serenity::CreateActionRow::Buttons(vec![
        serenity::CreateButton::new("music_pause")
            .emoji('⏯')
            .style(serenity::ButtonStyle::Primary),
        serenity::CreateButton::new("music_skip")
            .emoji('⏭')
            .style(serenity::ButtonStyle::Secondary),
        serenity::CreateButton::new("music_repeat")
            .emoji('🔁')
            .style(serenity::ButtonStyle::Secondary),
        serenity::CreateButton::new("music_shuffle")
            .emoji('🔀')
            .style(serenity::ButtonStyle::Secondary),
        serenity::CreateButton::new("music_stop")
            .emoji('⏹')
            .style(serenity::ButtonStyle::Danger),
    ])
}
