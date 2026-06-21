use crate::models::resource::*;
use crate::services::global;

fn manager() -> &'static crate::services::resource_manager::ResourceManager {
    global::resource_manager()
}

#[tauri::command]
pub async fn search_resources(
    query: String,
    game_version: Option<String>,
    loader: Option<String>,
    project_type: Option<String>,
    source: Option<String>,
    offset: u32,
    limit: u32,
) -> Result<Vec<ResourceSearchResult>, String> {
    let req = SearchRequest {
        query,
        game_version,
        loader,
        project_type,
        source,
        offset,
        limit,
    };
    manager().search(req).await
}

#[tauri::command]
pub async fn get_resource_detail(
    source: String,
    project_id: String,
) -> Result<ResourceDetail, String> {
    manager().get_detail(&source, &project_id).await
}

#[tauri::command]
pub async fn get_resource_versions(
    source: String,
    project_id: String,
    game_version: Option<String>,
    loader: Option<String>,
) -> Result<Vec<ResourceVersion>, String> {
    manager()
        .get_versions(&source, &project_id, game_version.as_deref(), loader.as_deref())
        .await
}

#[tauri::command]
pub async fn install_resource(
    server_id: String,
    source: String,
    project_id: String,
    version_id: String,
) -> Result<InstalledResource, String> {
    manager()
        .install_resource(&server_id, &source, &project_id, &version_id)
        .await
}

#[tauri::command]
pub fn get_installed_resources(server_id: String) -> Result<Vec<InstalledResource>, String> {
    manager().get_installed(&server_id)
}

#[tauri::command]
pub fn uninstall_resource(server_id: String, resource_id: String) -> Result<(), String> {
    manager().uninstall_resource(&server_id, &resource_id)
}

#[tauri::command]
pub async fn check_resource_updates(server_id: String) -> Result<Vec<ResourceUpdateInfo>, String> {
    manager().check_updates(&server_id).await
}

#[tauri::command]
pub async fn update_resource(
    server_id: String,
    resource_id: String,
    version_id: String,
) -> Result<InstalledResource, String> {
    manager()
        .update_resource(&server_id, &resource_id, &version_id)
        .await
}
