use crate::models::backup::*;
use crate::services::global;

fn manager() -> &'static crate::services::backup_manager::BackupManager {
    global::backup_manager()
}

#[tauri::command]
pub fn create_backup(
    server_id: String,
    name: Option<String>,
    incremental: bool,
    include_paths: Vec<String>,
) -> Result<BackupMeta, String> {
    let req = CreateBackupRequest {
        server_id,
        name,
        incremental,
        include_paths,
    };
    manager().create_backup(req)
}

#[tauri::command]
pub fn list_backups(server_id: String) -> Result<Vec<BackupMeta>, String> {
    manager().list_backups(&server_id)
}

#[tauri::command]
pub fn get_backup_detail(server_id: String, backup_id: String) -> Result<BackupManifest, String> {
    manager().get_backup_detail(&server_id, &backup_id)
}

#[tauri::command]
pub fn restore_backup(server_id: String, backup_id: String) -> Result<(), String> {
    manager().restore_backup(&server_id, &backup_id)
}

#[tauri::command]
pub fn delete_backup(server_id: String, backup_id: String) -> Result<(), String> {
    manager().delete_backup(&server_id, &backup_id)
}
