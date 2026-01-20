use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Option<i64>,
    pub platform: String,
    pub channel_id: String,
    pub channel_name: String,
    pub enabled: bool,
    pub poll_interval: i32,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub id: Option<i64>,
    pub channel_id: i64,
    pub stream_id: String,
    pub title: Option<String>,
    pub category: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    pub id: Option<i64>,
    pub stream_id: i64,
    pub collected_at: String,
    pub viewer_count: Option<i32>,
    pub chat_rate_1min: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelWithStats {
    #[serde(flatten)]
    pub channel: Channel,
    pub is_live: bool,
    pub current_viewers: Option<i32>,
    pub current_title: Option<String>,
}
