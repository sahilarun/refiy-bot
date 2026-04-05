pub enum HearthKeys {
    Player,
    Locale,
    Prefix,
    VoiceStatus,
}

impl HearthKeys {
    pub fn as_str(&self) -> &'static str {
        match self {
            HearthKeys::Player => "guild:player",
            HearthKeys::Locale => "guild:locale",
            HearthKeys::Prefix => "guild:prefix",
            HearthKeys::VoiceStatus => "guild:voice_status",
        }
    }
}

impl std::fmt::Display for HearthKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
