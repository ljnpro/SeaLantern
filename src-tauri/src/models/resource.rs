use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceSource {
    Modrinth,
    CurseForge,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    Mod,
    Plugin,
    DataPack,
    Shader,
    ResourcePack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSearchResult {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub summary: String,
    pub author: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub source: ResourceSource,
    pub project_type: ResourceType,
    pub categories: Vec<String>,
    pub latest_game_versions: Vec<String>,
    pub date_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceVersion {
    pub id: String,
    pub version_number: String,
    pub name: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub download_url: String,
    pub file_name: String,
    pub file_size: u64,
    pub dependencies: Vec<ResourceDependency>,
    pub date_published: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDependency {
    pub project_id: String,
    pub dependency_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDetail {
    pub id: String,
    pub name: String,
    pub description: String,
    pub source: ResourceSource,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub source_url: Option<String>,
    pub wiki_url: Option<String>,
    pub versions: Vec<ResourceVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledResource {
    pub id: String,
    pub project_id: String,
    pub source: ResourceSource,
    pub name: String,
    pub version_id: String,
    pub version_number: String,
    pub file_name: String,
    pub installed_at: u64,
    pub server_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUpdateInfo {
    pub resource_id: String,
    pub project_id: String,
    pub current_version: String,
    pub latest_version_id: String,
    pub latest_version_number: String,
    pub source: ResourceSource,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub game_version: Option<String>,
    pub loader: Option<String>,
    pub project_type: Option<String>,
    pub source: Option<String>,
    pub offset: u32,
    pub limit: u32,
}
