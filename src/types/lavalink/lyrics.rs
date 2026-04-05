use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lyrics {
    pub provider: String,
    pub sourceName: String,
    pub lines: Vec<LyricsLine>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LyricsLine {
    pub timestamp: u64,
    pub duration: u64,
    pub line: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LyricsSearchResponse {
    pub provider: String,
    pub sourceName: String,
    pub text: Option<String>,
    pub lines: Option<Vec<LyricsLine>>,
}
