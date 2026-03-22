use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    RestartServer,
    RunBackup {
        incremental: bool,
        include_paths: Vec<String>,
    },
    ExecuteCommand {
        command: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskSchedule {
    Interval { seconds: u64 },
    Daily { hour: u8, minute: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub enabled: bool,
    pub task_type: TaskType,
    pub schedule: TaskSchedule,
    pub created_at: u64,
    pub last_run_at: Option<u64>,
    pub last_run_result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub server_id: String,
    pub name: String,
    pub task_type: TaskType,
    pub schedule: TaskSchedule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionLog {
    pub id: String,
    pub task_id: String,
    pub executed_at: u64,
    pub result: String,
    pub duration_ms: u64,
}
