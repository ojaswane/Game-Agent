use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::time::{interval, sleep, timeout, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use crate::config::AgentConfig;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AgentMessage {
    Register {
        worker_id: String,
        max_sessions: usize,
    },
}

#[derive(Debug, Serialize)]
struct HeartbeatRequest {
    active_sessions: usize,
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
fn url_connection(server_url: &str) -> Result<String> {
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

fn heartbeat_endpoint(server_url: &str, worker_id: &str) -> Result<String> {
    let mut endpoint = Url::parse(server_url)?;
    match endpoint.scheme() {
        "http" | "https" => {}
        "ws" => endpoint.set_scheme("http").map_err(|_| anyhow!("Invalid worker server URL"))?,
        "wss" => endpoint.set_scheme("https").map_err(|_| anyhow!("Invalid worker server URL"))?,
        scheme => return Err(anyhow!("Unsupported server URL scheme for heartbeat: {scheme}")),
    }

    endpoint.set_query(None);
    endpoint.set_fragment(None);
    endpoint
        .path_segments_mut()
        .map_err(|_| anyhow!("Worker server URL cannot contain path segments"))?
        .pop_if_empty()
        .push("workers")
        .push(worker_id)
        .push("heartbeat");

    Ok(endpoint.into())
}

pub async fn run(config: AgentConfig) -> Result<()> {
    let ws_url = url_connection(&config.server_url)?;

    loop {
        println!("Connected to the server");

        match connect_and_run(&config, &ws_url).await {
            Ok(()) => {
                println!("WebSocket connection closed.");
            }
            Err(error) => {
                eprintln!("Agent network error: {error:#}");
            }
        }

        println!("Reconnecting in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}

async fn connect_and_run(
    config: &AgentConfig,
    ws_url: &str,
) -> Result<()> {
    let (ws_stream, _) = connect_async(ws_url).await?;
    let heartbeat_url = heartbeat_endpoint(&config.server_url, &config.worker_id)?;
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;

    println!("Connected to server!");

    let (mut write, mut read) = ws_stream.split();

    // Identify this worker to the server.
    let registration = AgentMessage::Register {
        worker_id: config.worker_id.clone(),
        max_sessions: config.max_sessions,
    };

    write
        .send(Message::Text(
            serde_json::to_string(&registration)?.into(),
        ))
        .await?;

    println!("Registration message sent.");

    let mut heartbeat = interval(Duration::from_secs(10));

    // The first interval tick is immediate; skip it because
    // registration has just been sent.
    heartbeat.tick().await;

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                let payload = HeartbeatRequest {
                    active_sessions: 0,
                };

                match timeout(
                    Duration::from_secs(4),
                    http_client.post(&heartbeat_url).json(&payload).send(),
                ).await {
                    Ok(Ok(response)) => match response.error_for_status() {
                        Ok(_) => println!("HTTP heartbeat sent."),
                        Err(error) => eprintln!("HTTP heartbeat failed: {error}"),
                    },
                    Ok(Err(error)) => eprintln!("HTTP heartbeat failed: {error}"),
                    Err(_) => eprintln!("HTTP heartbeat timed out"),
                }
            }

            incoming = read.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        let raw = text.to_string();

                        match serde_json::from_str::<ServerMessage>(&raw) {
                            Ok(ServerMessage::Registered) => {
                                println!("Server confirmed registration.");
                            }

                            Ok(ServerMessage::StartSession {
                                session_id,
                                game_id,
                            }) => {
                                println!(
                                    "Received START_SESSION: \
                                    session={session_id}, game={game_id}"
                                );

                                println!("Session accepted. Direct WebRTC media is not wired yet; this worker needs signaling messages from the control server.");
                            }

                            Ok(ServerMessage::StopSession {
                                session_id,
                            }) => {
                                println!("Received STOP_SESSION: {session_id}");
                            }

                            Ok(ServerMessage::Error { message }) => {
                                eprintln!("Server error: {message}");
                            }

                            Err(error) => {
                                eprintln!(
                                    "Unrecognized server message: \
                                    {error}; payload={raw}"
                                );
                            }
                        }
                    }

                    Some(Ok(Message::Ping(payload))) => {
                        write.send(Message::Pong(payload)).await?;
                    }

                    Some(Ok(Message::Pong(_))) => {}

                    Some(Ok(Message::Close(_))) | None => {
                        return Err(anyhow!(
                            "WebSocket connection closed by server"
                        ));
                    }

                    Some(Ok(_)) => {}

                    Some(Err(error)) => {
                        return Err(error.into());
                    }
                }
            }
        }
    }
}
