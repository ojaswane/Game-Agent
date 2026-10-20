use std::env;

pub struct AgentConfig {
    pub server_url: String,
    pub worker_id: String,
    pub max_sessions: usize,
    /// RTMP base URL of the streaming host
    pub stream_publish_url: String,
    /// Optional DirectShow audio input name
    pub audio_device: Option<String>,
}

impl AgentConfig {
    pub fn from_env() -> Self {
        Self {
            server_url: env::var("SERVER_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:3000".to_string()),
            worker_id: env::var("WORKER_ID").unwrap_or_else(|_| "gaming-pc-01".to_string()),
            max_sessions: env::var("MAX_SESSIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1),
            stream_publish_url: env::var("STREAM_PUBLISH_URL")
                .unwrap_or_else(|_| "rtmp://127.0.0.1:1935/live".to_string()),
            audio_device: env::var("AUDIO_DEVICE")
                .ok()
                .filter(|v| !v.trim().is_empty()),
        }
    }
}
