use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupType {
    Full,
    Incremental,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMeta {
    pub id: String,
    pub server_id: String,
    pub name: String,
    pub backup_type: BackupType,
    pub created_at: u64,
    pub size_bytes: u64,
    pub file_count: u32,
    pub parent_id: Option<String>,
    pub paths_included: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupFileEntry {
    pub relative_path: String,
    pub sha256: String,
    pub size: u64,
    pub modified_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub meta: BackupMeta,
    pub files: Vec<BackupFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBackupRequest {
    pub server_id: String,
    pub name: Option<String>,
    pub incremental: bool,
    pub include_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreBackupRequest {
    pub server_id: String,
    pub backup_id: String,
    pub overwrite: bool,
}
