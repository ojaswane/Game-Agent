use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{interval, sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::config::AgentConfig;
use crate::streaming::StreamProcess;

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

    println!("Connected to CloudRig server!");

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
    let mut sessions: HashMap<String, StreamProcess> = HashMap::new();

    // The first interval tick is immediate; skip it because
    // registration has just been sent.
    heartbeat.tick().await;

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                let message = AgentMessage::Heartbeat {
                    worker_id: config.worker_id.clone(),
                    active_sessions: sessions.len(),
                };

                write
                    .send(Message::Text(
                        serde_json::to_string(&message)?.into(),
                    ))
                    .await?;

                println!("Heartbeat sent.");
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

                                if sessions.contains_key(&session_id) {
                                    eprintln!("Session {session_id} is already streaming");
                                    continue;
                                }

                                match StreamProcess::start(config, &session_id).await {
                                    Ok(stream) => {
                                        println!("Publishing session {session_id} to {}", stream.publish_url);
                                        sessions.insert(session_id, stream);
                                    }
                                    Err(error) => {
                                        eprintln!("Could not start stream for {session_id}: {error:#}");
                                    }
                                }
                            }

                            Ok(ServerMessage::StopSession {
                                session_id,
                            }) => {
                                match sessions.remove(&session_id) {
                                    Some(stream) => {
                                        if let Err(error) = stream.stop().await {
                                            eprintln!("Failed to stop stream {session_id}: {error:#}");
                                        } else {
                                            println!("Stopped stream for session {session_id}");
                                        }
                                    }
                                    None => println!("No active stream for session {session_id}"),
                                }
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
