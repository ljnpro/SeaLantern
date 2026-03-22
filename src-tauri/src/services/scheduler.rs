use crate::models::backup::CreateBackupRequest;
use crate::models::scheduler::*;
use crate::utils::path::get_app_data_dir;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SCHEDULED_TASKS_FILE: &str = "sea_lantern_scheduled_tasks.json";
const TASK_LOGS_FILE: &str = "sea_lantern_task_logs.json";
const MAX_LOGS_PER_TASK: usize = 100;

pub struct SchedulerService {
    tasks: Mutex<Vec<ScheduledTask>>,
    execution_logs: Mutex<Vec<TaskExecutionLog>>,
    cancel_tokens: Mutex<HashMap<String, tokio::sync::oneshot::Sender<()>>>,
}

impl SchedulerService {
    pub fn new() -> Self {
        let tasks = load_tasks();
        let logs = load_logs();
        SchedulerService {
            tasks: Mutex::new(tasks),
            execution_logs: Mutex::new(logs),
            cancel_tokens: Mutex::new(HashMap::new()),
        }
    }

    pub fn create_task(&self, req: CreateTaskRequest) -> Result<ScheduledTask, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let task = ScheduledTask {
            id: uuid::Uuid::new_v4().to_string(),
            server_id: req.server_id,
            name: req.name,
            enabled: true,
            task_type: req.task_type,
            schedule: req.schedule,
            created_at: now,
            last_run_at: None,
            last_run_result: None,
        };

        {
            let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
            tasks.push(task.clone());
            save_tasks(&tasks)?;
        }

        if task.enabled {
            self.spawn_task_runner(&task);
        }

        Ok(task)
    }

    pub fn update_task(&self, updated: ScheduledTask) -> Result<ScheduledTask, String> {
        // Cancel existing runner
        self.cancel_task_runner(&updated.id);

        {
            let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
            if let Some(task) = tasks.iter_mut().find(|t| t.id == updated.id) {
                *task = updated.clone();
            } else {
                return Err("Task not found".to_string());
            }
            save_tasks(&tasks)?;
        }

        if updated.enabled {
            self.spawn_task_runner(&updated);
        }

        Ok(updated)
    }

    pub fn delete_task(&self, task_id: &str) -> Result<(), String> {
        self.cancel_task_runner(task_id);

        let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
        tasks.retain(|t| t.id != task_id);
        save_tasks(&tasks)?;

        // Clean up logs
        let mut logs = self.execution_logs.lock().map_err(|e| e.to_string())?;
        logs.retain(|l| l.task_id != task_id);
        save_logs(&logs)?;

        Ok(())
    }

    pub fn list_tasks(&self, server_id: &str) -> Result<Vec<ScheduledTask>, String> {
        let tasks = self.tasks.lock().map_err(|e| e.to_string())?;
        Ok(tasks
            .iter()
            .filter(|t| t.server_id == server_id)
            .cloned()
            .collect())
    }

    pub fn toggle_task(&self, task_id: &str, enabled: bool) -> Result<(), String> {
        let mut tasks = self.tasks.lock().map_err(|e| e.to_string())?;
        let task = tasks
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| "Task not found".to_string())?;
        task.enabled = enabled;
        let task_clone = task.clone();
        save_tasks(&tasks)?;
        drop(tasks);

        if enabled {
            self.spawn_task_runner(&task_clone);
        } else {
            self.cancel_task_runner(task_id);
        }

        Ok(())
    }

    pub fn get_execution_logs(&self, task_id: &str) -> Result<Vec<TaskExecutionLog>, String> {
        let logs = self.execution_logs.lock().map_err(|e| e.to_string())?;
        Ok(logs
            .iter()
            .filter(|l| l.task_id == task_id)
            .cloned()
            .collect())
    }

    pub fn start_all_tasks(&self) {
        let tasks = match self.tasks.lock() {
            Ok(t) => t.clone(),
            Err(_) => return,
        };

        for task in &tasks {
            if task.enabled {
                self.spawn_task_runner(task);
            }
        }
    }

    pub fn run_task_now(&self, task_id: &str) -> Result<(), String> {
        let tasks = self.tasks.lock().map_err(|e| e.to_string())?;
        let task = tasks
            .iter()
            .find(|t| t.id == task_id)
            .ok_or_else(|| "Task not found".to_string())?
            .clone();
        drop(tasks);

        let task_id_owned = task.id.clone();
        tokio::spawn(async move {
            let result = execute_task_action(&task).await;

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let log_entry = TaskExecutionLog {
                id: uuid::Uuid::new_v4().to_string(),
                task_id: task_id_owned.clone(),
                executed_at: now,
                result: match &result {
                    Ok(()) => "success".to_string(),
                    Err(e) => e.clone(),
                },
                duration_ms: 0,
            };

            let scheduler = crate::services::global::scheduler();
            if let Ok(mut tasks) = scheduler.tasks.lock() {
                if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id_owned) {
                    t.last_run_at = Some(now);
                    t.last_run_result = Some(log_entry.result.clone());
                }
                let _ = save_tasks(&tasks);
            }
            if let Ok(mut logs) = scheduler.execution_logs.lock() {
                logs.push(log_entry);
                // Cap logs
                let task_logs_count = logs.iter().filter(|l| l.task_id == task_id_owned).count();
                if task_logs_count > MAX_LOGS_PER_TASK {
                    let excess = task_logs_count - MAX_LOGS_PER_TASK;
                    let mut removed = 0;
                    logs.retain(|l| {
                        if l.task_id == task_id_owned && removed < excess {
                            removed += 1;
                            false
                        } else {
                            true
                        }
                    });
                }
                let _ = save_logs(&logs);
            }
        });

        Ok(())
    }

    fn spawn_task_runner(&self, task: &ScheduledTask) {
        let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

        if let Ok(mut tokens) = self.cancel_tokens.lock() {
            // Cancel existing runner if any
            if let Some(old_tx) = tokens.remove(&task.id) {
                let _ = old_tx.send(());
            }
            tokens.insert(task.id.clone(), tx);
        }

        let task = task.clone();
        tokio::spawn(async move {
            loop {
                let wait_duration = compute_next_wait(&task.schedule);

                tokio::select! {
                    _ = tokio::time::sleep(wait_duration) => {
                        let task_id = task.id.clone();
                        let start = SystemTime::now();
                        let result = execute_task_action(&task).await;
                        let duration_ms = start.elapsed().unwrap_or_default().as_millis() as u64;

                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs();

                        let log_entry = TaskExecutionLog {
                            id: uuid::Uuid::new_v4().to_string(),
                            task_id: task_id.clone(),
                            executed_at: now,
                            result: match &result {
                                Ok(()) => "success".to_string(),
                                Err(e) => e.clone(),
                            },
                            duration_ms,
                        };

                        let scheduler = crate::services::global::scheduler();
                        if let Ok(mut tasks) = scheduler.tasks.lock() {
                            if let Some(t) = tasks.iter_mut().find(|t| t.id == task_id) {
                                t.last_run_at = Some(now);
                                t.last_run_result = Some(log_entry.result.clone());
                            }
                            let _ = save_tasks(&tasks);
                        }
                        if let Ok(mut logs) = scheduler.execution_logs.lock() {
                            logs.push(log_entry);
                            let _ = save_logs(&logs);
                        }
                    }
                    _ = &mut rx => {
                        break;
                    }
                }
            }
        });
    }

    fn cancel_task_runner(&self, task_id: &str) {
        if let Ok(mut tokens) = self.cancel_tokens.lock() {
            if let Some(tx) = tokens.remove(task_id) {
                let _ = tx.send(());
            }
        }
    }
}

fn compute_next_wait(schedule: &TaskSchedule) -> Duration {
    match schedule {
        TaskSchedule::Interval { seconds } => Duration::from_secs(*seconds),
        TaskSchedule::Daily { hour, minute } => {
            let now = chrono::Local::now();
            let target = now
                .date_naive()
                .and_hms_opt(*hour as u32, *minute as u32, 0)
                .unwrap_or_else(|| now.naive_local());
            let target = if target <= now.naive_local() {
                target + chrono::Duration::days(1)
            } else {
                target
            };
            let diff = target - now.naive_local();
            Duration::from_secs(diff.num_seconds().max(1) as u64)
        }
    }
}

async fn execute_task_action(task: &ScheduledTask) -> Result<(), String> {
    let server_manager = crate::services::global::server_manager();

    match &task.task_type {
        TaskType::RestartServer => {
            server_manager.stop_server(&task.server_id)?;
            // Wait for server to stop (poll status)
            for _ in 0..60 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let status = server_manager.get_server_status(&task.server_id);
                if status.status == crate::models::server::ServerStatus::Stopped {
                    break;
                }
            }
            server_manager.start_server(&task.server_id)?;
            Ok(())
        }
        TaskType::RunBackup {
            incremental,
            include_paths,
        } => {
            let backup_manager = crate::services::global::backup_manager();
            let req = CreateBackupRequest {
                server_id: task.server_id.clone(),
                name: Some(format!(
                    "scheduled-{}",
                    chrono::Local::now().format("%Y%m%d-%H%M%S")
                )),
                incremental: *incremental,
                include_paths: include_paths.clone(),
            };
            backup_manager.create_backup(req)?;
            Ok(())
        }
        TaskType::ExecuteCommand { command } => {
            server_manager.send_command(&task.server_id, command)?;
            Ok(())
        }
    }
}

fn tasks_file_path() -> std::path::PathBuf {
    get_app_data_dir().join(SCHEDULED_TASKS_FILE)
}

fn logs_file_path() -> std::path::PathBuf {
    get_app_data_dir().join(TASK_LOGS_FILE)
}

fn load_tasks() -> Vec<ScheduledTask> {
    let path = tasks_file_path();
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_tasks(tasks: &[ScheduledTask]) -> Result<(), String> {
    let path = tasks_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let content =
        serde_json::to_string_pretty(tasks).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Failed to write: {}", e))
}

fn load_logs() -> Vec<TaskExecutionLog> {
    let path = logs_file_path();
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_logs(logs: &[TaskExecutionLog]) -> Result<(), String> {
    let path = logs_file_path();
    let content =
        serde_json::to_string_pretty(logs).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Failed to write: {}", e))
}
