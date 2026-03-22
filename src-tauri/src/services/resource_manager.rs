use crate::models::resource::*;
use crate::utils::path::get_app_data_dir;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const INSTALLED_RESOURCES_FILE: &str = "sea_lantern_installed_resources.json";
const MODRINTH_API_BASE: &str = "https://api.modrinth.com/v2";
const CURSEFORGE_API_BASE: &str = "https://api.curseforge.com/v1";
const MINECRAFT_GAME_ID: u32 = 432;

pub struct ResourceManager {
    client: reqwest::Client,
    installed: Mutex<Vec<InstalledResource>>,
}

impl ResourceManager {
    pub fn new() -> Result<Self, String> {
        let client = reqwest::Client::builder()
            .user_agent("SeaLantern/1.0 (Minecraft Server Manager)")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let installed = load_installed();

        Ok(ResourceManager {
            client,
            installed: Mutex::new(installed),
        })
    }

    pub async fn search(&self, req: SearchRequest) -> Result<Vec<ResourceSearchResult>, String> {
        let source = req.source.as_deref().unwrap_or("all");
        let mut results = Vec::new();

        if source == "all" || source == "modrinth" {
            match self.search_modrinth(&req).await {
                Ok(mut r) => results.append(&mut r),
                Err(e) => eprintln!("Modrinth search error: {}", e),
            }
        }

        if source == "all" || source == "curseforge" {
            match self.search_curseforge(&req).await {
                Ok(mut r) => results.append(&mut r),
                Err(e) => eprintln!("CurseForge search error: {}", e),
            }
        }

        // Sort by downloads
        results.sort_by(|a, b| b.downloads.cmp(&a.downloads));
        Ok(results)
    }

    async fn search_modrinth(
        &self,
        req: &SearchRequest,
    ) -> Result<Vec<ResourceSearchResult>, String> {
        let mut facets = Vec::new();

        if let Some(ref game_version) = req.game_version {
            facets.push(format!("[\"versions:{}\"]", game_version));
        }
        if let Some(ref loader) = req.loader {
            facets.push(format!("[\"categories:{}\"]", loader));
        }
        if let Some(ref project_type) = req.project_type {
            facets.push(format!("[\"project_type:{}\"]", project_type));
        }

        let facets_str = if facets.is_empty() {
            String::new()
        } else {
            format!("&facets=[{}]", facets.join(","))
        };

        let url = format!(
            "{}/search?query={}&offset={}&limit={}{}",
            MODRINTH_API_BASE, req.query, req.offset, req.limit, facets_str
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Modrinth request failed: {}", e))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse Modrinth response: {}", e))?;

        let hits = body["hits"].as_array().ok_or("Invalid Modrinth response")?;
        let mut results = Vec::new();

        for hit in hits {
            let project_type = match hit["project_type"].as_str().unwrap_or("mod") {
                "mod" => ResourceType::Mod,
                "plugin" => ResourceType::Plugin,
                "datapack" => ResourceType::DataPack,
                "shader" => ResourceType::Shader,
                "resourcepack" => ResourceType::ResourcePack,
                _ => ResourceType::Mod,
            };

            results.push(ResourceSearchResult {
                id: hit["project_id"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                name: hit["title"].as_str().unwrap_or_default().to_string(),
                slug: hit["slug"].as_str().unwrap_or_default().to_string(),
                summary: hit["description"].as_str().unwrap_or_default().to_string(),
                author: hit["author"].as_str().unwrap_or_default().to_string(),
                downloads: hit["downloads"].as_u64().unwrap_or(0),
                icon_url: hit["icon_url"].as_str().map(|s| s.to_string()),
                source: ResourceSource::Modrinth,
                project_type,
                categories: hit["categories"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                latest_game_versions: hit["versions"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                date_modified: hit["date_modified"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            });
        }

        Ok(results)
    }

    async fn search_curseforge(
        &self,
        req: &SearchRequest,
    ) -> Result<Vec<ResourceSearchResult>, String> {
        let api_key = crate::services::global::settings_manager()
            .get()
            .curseforge_api_key
            .clone();
        if api_key.is_empty() {
            return Ok(Vec::new());
        }

        let class_id = match req.project_type.as_deref() {
            Some("plugin") => 5,
            Some("resourcepack") => 12,
            _ => 6, // mods
        };

        let mut url = format!(
            "{}/mods/search?gameId={}&classId={}&searchFilter={}&index={}&pageSize={}",
            CURSEFORGE_API_BASE, MINECRAFT_GAME_ID, class_id, req.query, req.offset, req.limit
        );

        if let Some(ref game_version) = req.game_version {
            url.push_str(&format!("&gameVersion={}", game_version));
        }

        let resp = self
            .client
            .get(&url)
            .header("x-api-key", &api_key)
            .send()
            .await
            .map_err(|e| format!("CurseForge request failed: {}", e))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse CurseForge response: {}", e))?;

        let data = body["data"]
            .as_array()
            .ok_or("Invalid CurseForge response")?;
        let mut results = Vec::new();

        for item in data {
            let project_type = match item["classId"].as_u64().unwrap_or(6) {
                5 => ResourceType::Plugin,
                12 => ResourceType::ResourcePack,
                _ => ResourceType::Mod,
            };

            results.push(ResourceSearchResult {
                id: item["id"].as_u64().unwrap_or(0).to_string(),
                name: item["name"].as_str().unwrap_or_default().to_string(),
                slug: item["slug"].as_str().unwrap_or_default().to_string(),
                summary: item["summary"].as_str().unwrap_or_default().to_string(),
                author: item["authors"]
                    .as_array()
                    .and_then(|a| a.first())
                    .and_then(|a| a["name"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                downloads: item["downloadCount"].as_u64().unwrap_or(0),
                icon_url: item["logo"]["url"].as_str().map(|s| s.to_string()),
                source: ResourceSource::CurseForge,
                project_type,
                categories: item["categories"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v["name"].as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                latest_game_versions: item["latestFilesIndexes"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v["gameVersion"].as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                date_modified: item["dateModified"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            });
        }

        Ok(results)
    }

    pub async fn get_detail(
        &self,
        source: &str,
        project_id: &str,
    ) -> Result<ResourceDetail, String> {
        match source {
            "modrinth" => self.get_modrinth_detail(project_id).await,
            "curseforge" => self.get_curseforge_detail(project_id).await,
            _ => Err("Unknown source".to_string()),
        }
    }

    async fn get_modrinth_detail(&self, project_id: &str) -> Result<ResourceDetail, String> {
        let url = format!("{}/project/{}", MODRINTH_API_BASE, project_id);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let versions = self
            .get_versions("modrinth", project_id, None, None)
            .await?;

        Ok(ResourceDetail {
            id: body["id"].as_str().unwrap_or_default().to_string(),
            name: body["title"].as_str().unwrap_or_default().to_string(),
            description: body["body"].as_str().unwrap_or_default().to_string(),
            source: ResourceSource::Modrinth,
            downloads: body["downloads"].as_u64().unwrap_or(0),
            icon_url: body["icon_url"].as_str().map(|s| s.to_string()),
            source_url: body["source_url"].as_str().map(|s| s.to_string()),
            wiki_url: body["wiki_url"].as_str().map(|s| s.to_string()),
            versions,
        })
    }

    async fn get_curseforge_detail(&self, project_id: &str) -> Result<ResourceDetail, String> {
        let api_key = crate::services::global::settings_manager()
            .get()
            .curseforge_api_key
            .clone();
        if api_key.is_empty() {
            return Err("CurseForge API key not configured".to_string());
        }

        let url = format!("{}/mods/{}", CURSEFORGE_API_BASE, project_id);
        let resp = self
            .client
            .get(&url)
            .header("x-api-key", &api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let data = &body["data"];
        let versions = self
            .get_versions("curseforge", project_id, None, None)
            .await?;

        Ok(ResourceDetail {
            id: data["id"].as_u64().unwrap_or(0).to_string(),
            name: data["name"].as_str().unwrap_or_default().to_string(),
            description: data["summary"].as_str().unwrap_or_default().to_string(),
            source: ResourceSource::CurseForge,
            downloads: data["downloadCount"].as_u64().unwrap_or(0),
            icon_url: data["logo"]["url"].as_str().map(|s| s.to_string()),
            source_url: data["links"]["sourceUrl"].as_str().map(|s| s.to_string()),
            wiki_url: data["links"]["wikiUrl"].as_str().map(|s| s.to_string()),
            versions,
        })
    }

    pub async fn get_versions(
        &self,
        source: &str,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ResourceVersion>, String> {
        match source {
            "modrinth" => {
                self.get_modrinth_versions(project_id, game_version, loader)
                    .await
            }
            "curseforge" => {
                self.get_curseforge_versions(project_id, game_version)
                    .await
            }
            _ => Err("Unknown source".to_string()),
        }
    }

    async fn get_modrinth_versions(
        &self,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> Result<Vec<ResourceVersion>, String> {
        let mut url = format!("{}/project/{}/version", MODRINTH_API_BASE, project_id);
        let mut params = Vec::new();
        if let Some(gv) = game_version {
            params.push(format!("game_versions=[\"{}\"]", gv));
        }
        if let Some(l) = loader {
            params.push(format!("loaders=[\"{}\"]", l));
        }
        if !params.is_empty() {
            url.push_str(&format!("?{}", params.join("&")));
        }

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        let body: Vec<serde_json::Value> = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let mut versions = Vec::new();
        for v in &body {
            let primary_file = v["files"]
                .as_array()
                .and_then(|files| files.iter().find(|f| f["primary"].as_bool().unwrap_or(false)))
                .or_else(|| v["files"].as_array().and_then(|f| f.first()));

            let (download_url, file_name, file_size) = if let Some(file) = primary_file {
                (
                    file["url"].as_str().unwrap_or_default().to_string(),
                    file["filename"].as_str().unwrap_or_default().to_string(),
                    file["size"].as_u64().unwrap_or(0),
                )
            } else {
                continue;
            };

            versions.push(ResourceVersion {
                id: v["id"].as_str().unwrap_or_default().to_string(),
                version_number: v["version_number"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                name: v["name"].as_str().unwrap_or_default().to_string(),
                game_versions: v["game_versions"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                loaders: v["loaders"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|x| x.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                download_url,
                file_name,
                file_size,
                dependencies: v["dependencies"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|d| {
                                Some(ResourceDependency {
                                    project_id: d["project_id"]
                                        .as_str()
                                        .unwrap_or_default()
                                        .to_string(),
                                    dependency_type: d["dependency_type"]
                                        .as_str()
                                        .unwrap_or("optional")
                                        .to_string(),
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                date_published: v["date_published"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            });
        }

        Ok(versions)
    }

    async fn get_curseforge_versions(
        &self,
        project_id: &str,
        game_version: Option<&str>,
    ) -> Result<Vec<ResourceVersion>, String> {
        let api_key = crate::services::global::settings_manager()
            .get()
            .curseforge_api_key
            .clone();
        if api_key.is_empty() {
            return Err("CurseForge API key not configured".to_string());
        }

        let mut url = format!("{}/mods/{}/files", CURSEFORGE_API_BASE, project_id);
        if let Some(gv) = game_version {
            url.push_str(&format!("?gameVersion={}", gv));
        }

        let resp = self
            .client
            .get(&url)
            .header("x-api-key", &api_key)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let data = body["data"]
            .as_array()
            .ok_or("Invalid CurseForge response")?;
        let mut versions = Vec::new();

        for file in data {
            versions.push(ResourceVersion {
                id: file["id"].as_u64().unwrap_or(0).to_string(),
                version_number: file["displayName"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                name: file["displayName"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                game_versions: file["gameVersions"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                loaders: Vec::new(),
                download_url: file["downloadUrl"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                file_name: file["fileName"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                file_size: file["fileLength"].as_u64().unwrap_or(0),
                dependencies: file["dependencies"]
                    .as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|d| {
                                Some(ResourceDependency {
                                    project_id: d["modId"]
                                        .as_u64()
                                        .unwrap_or(0)
                                        .to_string(),
                                    dependency_type: match d["relationType"].as_u64().unwrap_or(0) {
                                        3 => "required".to_string(),
                                        _ => "optional".to_string(),
                                    },
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default(),
                date_published: file["fileDate"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            });
        }

        Ok(versions)
    }

    pub async fn install_resource(
        &self,
        server_id: &str,
        source: &str,
        project_id: &str,
        version_id: &str,
    ) -> Result<InstalledResource, String> {
        // Get version info
        let versions = self.get_versions(source, project_id, None, None).await?;
        let version = versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| "Version not found".to_string())?;

        // Get server info to determine target directory
        let server_manager = crate::services::global::server_manager();
        let servers = server_manager.get_server_list();
        let server = servers
            .iter()
            .find(|s| s.id == server_id)
            .ok_or_else(|| "Server not found".to_string())?;

        let target_subdir = determine_target_dir(&server.core_type);
        let target_dir = Path::new(&server.path).join(target_subdir);
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create target directory: {}", e))?;

        // Download the file
        let resp = self
            .client
            .get(&version.download_url)
            .send()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("Failed to read download: {}", e))?;

        let target_path = target_dir.join(&version.file_name);
        std::fs::write(&target_path, &bytes)
            .map_err(|e| format!("Failed to write file: {}", e))?;

        // Get project name
        let detail = self.get_detail(source, project_id).await?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let installed = InstalledResource {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: project_id.to_string(),
            source: if source == "modrinth" {
                ResourceSource::Modrinth
            } else {
                ResourceSource::CurseForge
            },
            name: detail.name,
            version_id: version_id.to_string(),
            version_number: version.version_number.clone(),
            file_name: version.file_name.clone(),
            installed_at: now,
            server_id: server_id.to_string(),
        };

        {
            let mut list = self.installed.lock().map_err(|e| e.to_string())?;
            list.push(installed.clone());
            save_installed(&list)?;
        }

        Ok(installed)
    }

    pub fn get_installed(&self, server_id: &str) -> Result<Vec<InstalledResource>, String> {
        let list = self.installed.lock().map_err(|e| e.to_string())?;
        Ok(list
            .iter()
            .filter(|r| r.server_id == server_id)
            .cloned()
            .collect())
    }

    pub fn uninstall_resource(
        &self,
        server_id: &str,
        resource_id: &str,
    ) -> Result<(), String> {
        let mut list = self.installed.lock().map_err(|e| e.to_string())?;

        let resource = list
            .iter()
            .find(|r| r.id == resource_id && r.server_id == server_id)
            .ok_or_else(|| "Resource not found".to_string())?
            .clone();

        // Delete the file
        let server_manager = crate::services::global::server_manager();
        let servers = server_manager.get_server_list();
        if let Some(server) = servers.iter().find(|s| s.id == server_id) {
            let target_subdir = determine_target_dir(&server.core_type);
            let file_path = Path::new(&server.path)
                .join(target_subdir)
                .join(&resource.file_name);
            if file_path.exists() {
                std::fs::remove_file(&file_path)
                    .map_err(|e| format!("Failed to delete file: {}", e))?;
            }
        }

        list.retain(|r| r.id != resource_id);
        save_installed(&list)?;

        Ok(())
    }

    pub async fn check_updates(
        &self,
        server_id: &str,
    ) -> Result<Vec<ResourceUpdateInfo>, String> {
        let installed = self.get_installed(server_id)?;
        let mut updates = Vec::new();

        // Get server info for game version
        let server_manager = crate::services::global::server_manager();
        let servers = server_manager.get_server_list();
        let server = servers
            .iter()
            .find(|s| s.id == server_id)
            .ok_or_else(|| "Server not found".to_string())?;

        for resource in &installed {
            let source_str = match resource.source {
                ResourceSource::Modrinth => "modrinth",
                ResourceSource::CurseForge => "curseforge",
            };

            if let Ok(versions) = self
                .get_versions(
                    source_str,
                    &resource.project_id,
                    Some(&server.mc_version),
                    None,
                )
                .await
            {
                if let Some(latest) = versions.first() {
                    if latest.id != resource.version_id {
                        updates.push(ResourceUpdateInfo {
                            resource_id: resource.id.clone(),
                            project_id: resource.project_id.clone(),
                            current_version: resource.version_number.clone(),
                            latest_version_id: latest.id.clone(),
                            latest_version_number: latest.version_number.clone(),
                            source: resource.source.clone(),
                            name: resource.name.clone(),
                        });
                    }
                }
            }
        }

        Ok(updates)
    }

    pub async fn update_resource(
        &self,
        server_id: &str,
        resource_id: &str,
        version_id: &str,
    ) -> Result<InstalledResource, String> {
        // Get the current installed resource
        let (project_id, source_str, old_file_name) = {
            let list = self.installed.lock().map_err(|e| e.to_string())?;
            let resource = list
                .iter()
                .find(|r| r.id == resource_id && r.server_id == server_id)
                .ok_or_else(|| "Resource not found".to_string())?;
            (
                resource.project_id.clone(),
                match resource.source {
                    ResourceSource::Modrinth => "modrinth",
                    ResourceSource::CurseForge => "curseforge",
                }
                .to_string(),
                resource.file_name.clone(),
            )
        };

        // Remove old file
        let server_manager = crate::services::global::server_manager();
        let servers = server_manager.get_server_list();
        if let Some(server) = servers.iter().find(|s| s.id == server_id) {
            let target_subdir = determine_target_dir(&server.core_type);
            let old_path = Path::new(&server.path)
                .join(target_subdir)
                .join(&old_file_name);
            if old_path.exists() {
                let _ = std::fs::remove_file(&old_path);
            }
        }

        // Remove old entry
        {
            let mut list = self.installed.lock().map_err(|e| e.to_string())?;
            list.retain(|r| r.id != resource_id);
            save_installed(&list)?;
        }

        // Install new version
        self.install_resource(server_id, &source_str, &project_id, version_id)
            .await
    }
}

fn determine_target_dir(core_type: &str) -> &str {
    let lower = core_type.to_lowercase();
    if lower.contains("fabric")
        || lower.contains("forge")
        || lower.contains("neoforge")
        || lower.contains("quilt")
    {
        "mods"
    } else {
        "plugins"
    }
}

fn installed_file_path() -> std::path::PathBuf {
    get_app_data_dir().join(INSTALLED_RESOURCES_FILE)
}

fn load_installed() -> Vec<InstalledResource> {
    let path = installed_file_path();
    if !path.exists() {
        return Vec::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_installed(list: &[InstalledResource]) -> Result<(), String> {
    let path = installed_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let content =
        serde_json::to_string_pretty(list).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Failed to write: {}", e))
}
