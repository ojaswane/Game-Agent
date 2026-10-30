use std::env;

pub struct AgentConfig {
    pub server_url: String,
    pub worker_id: String,
    pub max_sessions: usize,
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
        }
    }
}
