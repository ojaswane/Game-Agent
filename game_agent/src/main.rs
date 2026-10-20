mod config;
mod worker;

use config::AgentConfig;
use worker::Worker;
mod network;
-O
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AgentConfig::from_env();

    let worker = Worker::new(
        config.worker_id.clone(),
        config.max_sessions,
    );

    println!("CloudRig Agent starting...");
    println!("Worker ID: {}", worker.id);
    println!("Server URL: {}", config.server_url);
    println!("Maximum sessions: {}", worker.max_sessions);


    network::run(config).await?;
    Ok(())
}
