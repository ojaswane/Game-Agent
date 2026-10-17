use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct worker {
    pub id: string
    pub status: WorkerStatus,
    pub max_sessions: usize,
    pub active_sessions: usize
}


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus{
    online,
    offline
}

impl worker {
    pub fn new(id: String, max_sessions: usize) -> Self {
        Self{
            id ,
            status : WorkerStatus:online,
            max_sessions,
            active_sessions: 0,
        }
    }
}