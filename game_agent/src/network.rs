use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::time::{interval, sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::config::AgentConfig;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AgentMessage {
    Register {
        worker_id: String,
        max_sessions: usize,
    },

    Heartbeat {
        worker_id: String,
        active_sessions: usize,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage {
    Registered,
    StartSession { session_id: String, game_id: String },
    StopSession { session_id: String },
    Error { message: String },
}

// Make the connection
pub fn url_connection(server_url: &str) -> Result<Sting> {
    let base = server_url.trim_end_matches('/');

    if let Some(rest) = base.strip_prefix("https://") {
        Ok(format!("wss://{rest}/ws/worker"))
    } else if let Some(rest) = base.strip_prefix("http://") {
        Ok(format!("ws://{rest}/ws/worker"))
    } else if base.starts_with("ws://") || base.starts_with("wss://") {
        Ok(format!("{base}/ws/worker"))
    } else {
        Err(anyhow!("Unsupported server URL: {base}"))
    }
}
