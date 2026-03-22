use crate::models::backup::*;
use crate::utils::path::get_app_data_dir;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct BackupManager {
    manifests: Mutex<HashMap<String, Vec<BackupMeta>>>,
}

impl BackupManager {
    pub fn new() -> Self {
        BackupManager {
            manifests: Mutex::new(HashMap::new()),
        }
    }

    fn backups_dir(&self) -> PathBuf {
        get_app_data_dir().join("backups")
    }

    fn server_backup_dir(&self, server_id: &str) -> PathBuf {
        self.backups_dir().join(server_id)
    }

    fn backup_dir(&self, server_id: &str, backup_id: &str) -> PathBuf {
        self.server_backup_dir(server_id).join(backup_id)
    }

    fn load_manifest(&self, server_id: &str, backup_id: &str) -> Result<BackupManifest, String> {
        let manifest_path = self.backup_dir(server_id, backup_id).join("manifest.json");
        let content = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))
    }

    fn save_manifest(&self, server_id: &str, manifest: &BackupManifest) -> Result<(), String> {
        let backup_path = self.backup_dir(server_id, &manifest.meta.id);
        std::fs::create_dir_all(&backup_path)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;
        let manifest_path = backup_path.join("manifest.json");
        let content = serde_json::to_string_pretty(manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        std::fs::write(&manifest_path, content)
            .map_err(|e| format!("Failed to write manifest: {}", e))
    }

    fn compute_sha256(path: &Path) -> Result<String, String> {
        let mut file =
            std::fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        loop {
            let n = file
                .read(&mut buffer)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn walk_directory(
        base: &Path,
        prefix: &Path,
    ) -> Result<Vec<(PathBuf, BackupFileEntry)>, String> {
        let mut entries = Vec::new();
        if !base.join(prefix).exists() {
            return Ok(entries);
        }
        let dir = std::fs::read_dir(base.join(prefix))
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in dir {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();
            let relative = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_path_buf();

            if path.is_dir() {
                entries.extend(Self::walk_directory(base, &relative)?);
            } else {
                let metadata = path
                    .metadata()
                    .map_err(|e| format!("Failed to read metadata: {}", e))?;
                let modified_at = metadata
                    .modified()
                    .unwrap_or(SystemTime::UNIX_EPOCH)
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let sha256 = Self::compute_sha256(&path)?;

                entries.push((
                    path.clone(),
                    BackupFileEntry {
                        relative_path: relative.to_string_lossy().to_string(),
                        sha256,
                        size: metadata.len(),
                        modified_at,
                    },
                ));
            }
        }
        Ok(entries)
    }

    pub fn create_backup(&self, req: CreateBackupRequest) -> Result<BackupMeta, String> {
        let server_manager = crate::services::global::server_manager();
        let servers = server_manager.get_server_list();
        let server = servers
            .iter()
            .find(|s| s.id == req.server_id)
            .ok_or_else(|| "Server not found".to_string())?;

        let server_path = Path::new(&server.path);
        let backup_id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let name = req.name.unwrap_or_else(|| {
            format!("backup-{}", chrono::Local::now().format("%Y%m%d-%H%M%S"))
        });

        let include_paths = if req.include_paths.is_empty() {
            vec!["world".to_string()]
        } else {
            req.include_paths
        };

        // Collect parent files for incremental comparison
        let parent_files: HashMap<String, String> = if req.incremental {
            let existing = self.list_backups(&req.server_id)?;
            if let Some(last) = existing.last() {
                let parent_manifest = self.load_manifest(&req.server_id, &last.id)?;
                parent_manifest
                    .files
                    .iter()
                    .map(|f| (f.relative_path.clone(), f.sha256.clone()))
                    .collect()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };

        let parent_id = if req.incremental {
            let existing = self.list_backups(&req.server_id)?;
            existing.last().map(|b| b.id.clone())
        } else {
            None
        };

        // Walk all included paths and collect files
        let mut all_files = Vec::new();
        let mut total_size: u64 = 0;

        for include_path in &include_paths {
            let entries = Self::walk_directory(server_path, Path::new(include_path))?;
            for (abs_path, file_entry) in entries {
                // For incremental: skip unchanged files
                if req.incremental {
                    if let Some(parent_hash) = parent_files.get(&file_entry.relative_path) {
                        if *parent_hash == file_entry.sha256 {
                            // File unchanged, still record in manifest but don't copy
                            all_files.push((None, file_entry));
                            continue;
                        }
                    }
                }
                total_size += file_entry.size;
                all_files.push((Some(abs_path), file_entry));
            }
        }

        let backup_data_dir = self.backup_dir(&req.server_id, &backup_id).join("data");
        std::fs::create_dir_all(&backup_data_dir)
            .map_err(|e| format!("Failed to create backup data directory: {}", e))?;

        // Copy files
        for (abs_path, file_entry) in &all_files {
            if let Some(src) = abs_path {
                let dest = backup_data_dir.join(&file_entry.relative_path);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }
                std::fs::copy(src, &dest)
                    .map_err(|e| format!("Failed to copy file: {}", e))?;
            }
        }

        let file_entries: Vec<BackupFileEntry> =
            all_files.into_iter().map(|(_, entry)| entry).collect();
        let file_count = file_entries.len() as u32;

        let meta = BackupMeta {
            id: backup_id.clone(),
            server_id: req.server_id.clone(),
            name,
            backup_type: if req.incremental {
                BackupType::Incremental
            } else {
                BackupType::Full
            },
            created_at: now,
            size_bytes: total_size,
            file_count,
            parent_id,
            paths_included: include_paths,
            notes: None,
        };

        let manifest = BackupManifest {
            meta: meta.clone(),
            files: file_entries,
        };

        self.save_manifest(&req.server_id, &manifest)?;

        // Update cache
        if let Ok(mut cache) = self.manifests.lock() {
            cache
                .entry(req.server_id)
                .or_default()
                .push(meta.clone());
        }

        Ok(meta)
    }

    pub fn list_backups(&self, server_id: &str) -> Result<Vec<BackupMeta>, String> {
        let server_dir = self.server_backup_dir(server_id);
        if !server_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();
        let entries = std::fs::read_dir(&server_dir)
            .map_err(|e| format!("Failed to read backup directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            if entry.path().is_dir() {
                let manifest_path = entry.path().join("manifest.json");
                if manifest_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = serde_json::from_str::<BackupManifest>(&content) {
                            backups.push(manifest.meta);
                        }
                    }
                }
            }
        }

        backups.sort_by_key(|b| b.created_at);
        Ok(backups)
    }

    pub fn get_backup_detail(
        &self,
        server_id: &str,
        backup_id: &str,
    ) -> Result<BackupManifest, String> {
        self.load_manifest(server_id, backup_id)
    }

    pub fn restore_backup(&self, server_id: &str, backup_id: &str) -> Result<(), String> {
        // Verify server is stopped
        let server_manager = crate::services::global::server_manager();
        let status = server_manager.get_server_status(server_id);
        if status.status != crate::models::server::ServerStatus::Stopped {
            return Err("Server must be stopped before restore".to_string());
        }

        let servers = server_manager.get_server_list();
        let server = servers
            .iter()
            .find(|s| s.id == server_id)
            .ok_or_else(|| "Server not found".to_string())?;

        let server_path = Path::new(&server.path);
        let manifest = self.load_manifest(server_id, backup_id)?;

        // For incremental backups, we need to chain back through parents
        let mut file_sources: HashMap<String, PathBuf> = HashMap::new();
        self.collect_file_sources(server_id, backup_id, &mut file_sources)?;

        // Restore files
        for (relative_path, source_path) in &file_sources {
            let dest = server_path.join(relative_path);
            if let Some(parent) = dest.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }
            std::fs::copy(source_path, &dest)
                .map_err(|e| format!("Failed to restore file {}: {}", relative_path, e))?;
        }

        // Also restore files that exist in manifest but were only in the current backup data
        let backup_data_dir = self.backup_dir(server_id, backup_id).join("data");
        for file_entry in &manifest.files {
            let data_file = backup_data_dir.join(&file_entry.relative_path);
            if data_file.exists() && !file_sources.contains_key(&file_entry.relative_path) {
                let dest = server_path.join(&file_entry.relative_path);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory: {}", e))?;
                }
                std::fs::copy(&data_file, &dest).map_err(|e| {
                    format!("Failed to restore file {}: {}", file_entry.relative_path, e)
                })?;
            }
        }

        Ok(())
    }

    fn collect_file_sources(
        &self,
        server_id: &str,
        backup_id: &str,
        sources: &mut HashMap<String, PathBuf>,
    ) -> Result<(), String> {
        let manifest = self.load_manifest(server_id, backup_id)?;
        let backup_data_dir = self.backup_dir(server_id, backup_id).join("data");

        // First, if there's a parent, collect from it (parent files as base)
        if let Some(ref parent_id) = manifest.meta.parent_id {
            self.collect_file_sources(server_id, parent_id, sources)?;
        }

        // Then override with files from this backup
        for file_entry in &manifest.files {
            let data_file = backup_data_dir.join(&file_entry.relative_path);
            if data_file.exists() {
                sources.insert(file_entry.relative_path.clone(), data_file);
            }
        }

        Ok(())
    }

    pub fn delete_backup(&self, server_id: &str, backup_id: &str) -> Result<(), String> {
        let backup_path = self.backup_dir(server_id, backup_id);
        if backup_path.exists() {
            std::fs::remove_dir_all(&backup_path)
                .map_err(|e| format!("Failed to delete backup: {}", e))?;
        }

        if let Ok(mut cache) = self.manifests.lock() {
            if let Some(list) = cache.get_mut(server_id) {
                list.retain(|b| b.id != backup_id);
            }
        }

        Ok(())
    }
}
