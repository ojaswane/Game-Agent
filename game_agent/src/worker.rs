use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Worker {
    pub id: String,
    pub status: WorkerStatus,
    pub max_sessions: usize,
    pub active_sessions: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    Online,
    Offline,
}

impl Worker {
    pub fn new(id: String, max_sessions: usize) -> Self {
        Self {
            id,
            status: WorkerStatus::Online,
            max_sessions,
            active_sessions: 0,
        }
    }
}
