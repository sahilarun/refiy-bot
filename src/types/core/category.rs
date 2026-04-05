#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum HearthCategory {
    Unknown = 0,
    Configurations = 1,
    Informations = 2,
    Music = 3,
    Filters = 4,
    Playlists = 5,
    Reports = 6,
    Developers = 7,
}

impl std::fmt::Display for HearthCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HearthCategory::Unknown => write!(f, "Unknown"),
            HearthCategory::Configurations => write!(f, "Configurations"),
            HearthCategory::Informations => write!(f, "Informations"),
            HearthCategory::Music => write!(f, "Music"),
            HearthCategory::Filters => write!(f, "Filters"),
            HearthCategory::Playlists => write!(f, "Playlists"),
            HearthCategory::Reports => write!(f, "Reports"),
            HearthCategory::Developers => write!(f, "Developers"),
        }
    }
}

impl From<u8> for HearthCategory {
    fn from(v: u8) -> Self {
        match v {
            1 => HearthCategory::Configurations,
            2 => HearthCategory::Informations,
            3 => HearthCategory::Music,
            4 => HearthCategory::Filters,
            5 => HearthCategory::Playlists,
            6 => HearthCategory::Reports,
            7 => HearthCategory::Developers,
            _ => HearthCategory::Unknown,
        }
    }
}
