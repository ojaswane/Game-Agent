use anyhow::{Result, anyhow};
use serde::Serialize;
use tokio::time::{Duration, sleep, timeout};
use url::Url;

use crate::config::AgentConfig;

#[derive(Debug, Serialize)]
struct RegisterWorkerRequest {
    id: String,
    max_sessions: usize,
}

#[derive(Debug, Serialize)]
struct HeartbeatRequest {
    active_sessions: usize,
}

fn worker_endpoint(server_url: &str, path: &[&str]) -> Result<String> {
    let mut endpoint = Url::parse(server_url)?;
    match endpoint.scheme() {
        "http" | "https" => {}
        "ws" => endpoint
            .set_scheme("http")
            .map_err(|_| anyhow!("Invalid worker server URL"))?,
        "wss" => endpoint
            .set_scheme("https")
            .map_err(|_| anyhow!("Invalid worker server URL"))?,
        scheme => return Err(anyhow!("Unsupported server URL scheme: {scheme}")),
    }

    endpoint.set_query(None);
    endpoint.set_fragment(None);
    endpoint
        .path_segments_mut()
        .map_err(|_| anyhow!("Worker server URL cannot contain path segments"))?
        .pop_if_empty()
        .extend(path.iter().copied());

    Ok(endpoint.into())
}

pub async fn run(config: AgentConfig) -> Result<()> {
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()?;
    let registration_url = worker_endpoint(&config.server_url, &["workers", "register"])?;
    let heartbeat_url = worker_endpoint(
        &config.server_url,
        &["workers", &config.worker_id, "heartbeat"],
    )?;

    let registration = RegisterWorkerRequest {
        id: config.worker_id.clone(),
        max_sessions: config.max_sessions,
    };

    loop {
        println!("Registering worker at {registration_url}...");
        match timeout(
            Duration::from_secs(4),
            http_client
                .post(&registration_url)
                .json(&registration)
                .send(),
        )
        .await
        {
            Ok(Ok(response)) => match response.error_for_status() {
                Ok(_) => {
                    println!("Worker registered successfully.");
                    break;
                }
                Err(error) => eprintln!("Worker registration failed: {error}"),
            },
            Ok(Err(error)) => eprintln!("Worker registration failed: {error}"),
            Err(_) => eprintln!("Worker registration timed out"),
        }

        println!("Retrying registration in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }

    let heartbeat = HeartbeatRequest { active_sessions: 0 };
    loop {
        sleep(Duration::from_secs(10)).await;

        match timeout(
            Duration::from_secs(4),
            http_client.post(&heartbeat_url).json(&heartbeat).send(),
        )
        .await
        {
            Ok(Ok(response)) => match response.error_for_status() {
                Ok(_) => println!("HTTP heartbeat sent."),
                Err(error) => eprintln!("HTTP heartbeat failed: {error}"),
            },
            Ok(Err(error)) => eprintln!("HTTP heartbeat failed: {error}"),
            Err(_) => eprintln!("HTTP heartbeat timed out"),
        }
    }
}
