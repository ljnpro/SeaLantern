use crate::models::scheduler::*;
use crate::services::global;

fn scheduler() -> &'static crate::services::scheduler::SchedulerService {
    global::scheduler()
}

#[tauri::command]
pub fn create_scheduled_task(
    server_id: String,
    name: String,
    task_type: serde_json::Value,
    schedule: serde_json::Value,
) -> Result<ScheduledTask, String> {
    let task_type: TaskType =
        serde_json::from_value(task_type).map_err(|e| format!("Invalid task_type: {}", e))?;
    let schedule: TaskSchedule =
        serde_json::from_value(schedule).map_err(|e| format!("Invalid schedule: {}", e))?;
    let req = CreateTaskRequest { server_id, name, task_type, schedule };
    scheduler().create_task(req)
}

#[tauri::command]
pub fn update_scheduled_task(task: serde_json::Value) -> Result<ScheduledTask, String> {
    let task: ScheduledTask =
        serde_json::from_value(task).map_err(|e| format!("Invalid task: {}", e))?;
    scheduler().update_task(task)
}

#[tauri::command]
pub fn delete_scheduled_task(task_id: String) -> Result<(), String> {
    scheduler().delete_task(&task_id)
}

#[tauri::command]
pub fn list_scheduled_tasks(server_id: String) -> Result<Vec<ScheduledTask>, String> {
    scheduler().list_tasks(&server_id)
}

#[tauri::command]
pub fn toggle_scheduled_task(task_id: String, enabled: bool) -> Result<(), String> {
    scheduler().toggle_task(&task_id, enabled)
}

#[tauri::command]
pub fn get_task_execution_logs(task_id: String) -> Result<Vec<TaskExecutionLog>, String> {
    scheduler().get_execution_logs(&task_id)
}

#[tauri::command]
pub fn run_task_now(task_id: String) -> Result<(), String> {
    scheduler().run_task_now(&task_id)
}
