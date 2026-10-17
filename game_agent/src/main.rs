mod config;
mod worker;

use config::AgentConfig;
use worker::Worker;

#[tokio::main]
async fn main() {
    let config = AgentConfig::from_env();

    let worker = Worker::new(
        config.worker_id.clone(),
        config.max_sessions,
    );

    println!("CloudRig Agent starting...");
    println!("Worker ID: {}", worker.id);
    println!("Server URL: {}", config.server_url);
    println!("Maximum sessions: {}", worker.max_sessions);
    println!("Active sessions: {}", worker.active_sessions);
    println!("Status: {:?}", worker.status);
}