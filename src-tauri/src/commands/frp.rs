use crate::models::frp::*;
use crate::services::global;

fn manager() -> &'static crate::services::frp_manager::FrpManager {
    global::frp_manager()
}

#[tauri::command]
pub async fn download_frpc() -> Result<FrpBinaryInfo, String> {
    manager().download_frpc().await
}

#[tauri::command]
pub fn get_frpc_info() -> Result<Option<FrpBinaryInfo>, String> {
    manager().get_frpc_info()
}

#[tauri::command]
pub fn save_frp_config(config: serde_json::Value) -> Result<(), String> {
    let config: FrpConfig =
        serde_json::from_value(config).map_err(|e| format!("Invalid config: {}", e))?;
    manager().save_config(config)
}

#[tauri::command]
pub fn get_frp_config(server_id: String) -> Result<Option<FrpConfig>, String> {
    manager().get_config(&server_id)
}

#[tauri::command]
pub fn delete_frp_config(server_id: String) -> Result<(), String> {
    manager().delete_config(&server_id)
}

#[tauri::command]
pub fn start_frpc(server_id: String) -> Result<(), String> {
    manager().start_frpc(&server_id)
}

#[tauri::command]
pub fn stop_frpc(server_id: String) -> Result<(), String> {
    manager().stop_frpc(&server_id)
}

#[tauri::command]
pub fn get_frp_status(server_id: String) -> Result<FrpStatus, String> {
    manager().get_status(&server_id)
}
