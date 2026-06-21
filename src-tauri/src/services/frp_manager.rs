use crate::models::frp::*;
use crate::utils::path::get_app_data_dir;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Child;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const FRP_CONFIGS_FILE: &str = "sea_lantern_frp_configs.json";
const FRP_DIR: &str = "frpc";

pub struct FrpManager {
    configs: Mutex<HashMap<String, FrpConfig>>,
    processes: Mutex<HashMap<String, Child>>,
    statuses: Mutex<HashMap<String, FrpStatus>>,
}

impl FrpManager {
    pub fn new() -> Self {
        let configs = load_configs();
        FrpManager {
            configs: Mutex::new(configs),
            processes: Mutex::new(HashMap::new()),
            statuses: Mutex::new(HashMap::new()),
        }
    }

    fn frpc_dir(&self) -> PathBuf {
        get_app_data_dir().join(FRP_DIR)
    }

    fn frpc_binary_path(&self) -> PathBuf {
        let dir = self.frpc_dir();
        if cfg!(windows) {
            dir.join("frpc.exe")
        } else {
            dir.join("frpc")
        }
    }

    pub async fn download_frpc(&self) -> Result<FrpBinaryInfo, String> {
        let (os, arch) = detect_platform();
        let version = "0.61.1";
        let archive_name = format!("frp_{}_{}_{}", version, os, arch);
        let ext = if os == "windows" { "zip" } else { "tar.gz" };
        let url = format!(
            "https://github.com/fatedier/frp/releases/download/v{}/{}.{}",
            version, archive_name, ext
        );

        let client = reqwest::Client::builder()
            .user_agent("SeaLantern/1.0")
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| format!("Failed to create client: {}", e))?;

        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Download failed with status: {}", resp.status()));
        }

        let bytes = resp
            .bytes()
            .await
            .map_err(|e| format!("Failed to read download: {}", e))?;

        let frpc_dir = self.frpc_dir();
        std::fs::create_dir_all(&frpc_dir)
            .map_err(|e| format!("Failed to create frpc directory: {}", e))?;

        // Extract frpc binary from archive
        let temp_dir = frpc_dir.join("temp");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        if ext == "zip" {
            let cursor = std::io::Cursor::new(&bytes);
            let mut archive =
                zip::ZipArchive::new(cursor).map_err(|e| format!("Failed to open zip: {}", e))?;

            for i in 0..archive.len() {
                let mut file = archive
                    .by_index(i)
                    .map_err(|e| format!("Failed to read zip entry: {}", e))?;
                let name = file.name().to_string();
                if name.contains("frpc") && !name.ends_with('/') && !name.contains("frps") {
                    let target = self.frpc_binary_path();
                    let mut out = std::fs::File::create(&target)
                        .map_err(|e| format!("Failed to create file: {}", e))?;
                    std::io::copy(&mut file, &mut out)
                        .map_err(|e| format!("Failed to write file: {}", e))?;
                    break;
                }
            }
        } else {
            let cursor = std::io::Cursor::new(&bytes);
            let gz = flate2::read::GzDecoder::new(cursor);
            let mut archive = tar::Archive::new(gz);

            for entry in archive
                .entries()
                .map_err(|e| format!("Failed to read tar: {}", e))?
            {
                let mut entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry
                    .path()
                    .map_err(|e| format!("Failed to get path: {}", e))?
                    .to_path_buf();
                let name = path.to_string_lossy().to_string();
                if name.contains("frpc") && !name.contains("frps") && !name.ends_with('/') {
                    let target = self.frpc_binary_path();
                    let mut out = std::fs::File::create(&target)
                        .map_err(|e| format!("Failed to create file: {}", e))?;
                    std::io::copy(&mut entry, &mut out)
                        .map_err(|e| format!("Failed to write file: {}", e))?;

                    // Set executable on Unix
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o755))
                            .map_err(|e| format!("Failed to set permissions: {}", e))?;
                    }
                    break;
                }
            }
        }

        // Clean up temp
        let _ = std::fs::remove_dir_all(&temp_dir);

        let binary_path = self.frpc_binary_path();
        if !binary_path.exists() {
            return Err("Failed to extract frpc binary".to_string());
        }

        Ok(FrpBinaryInfo {
            version: version.to_string(),
            path: binary_path.to_string_lossy().to_string(),
            arch: format!("{}_{}", os, arch),
        })
    }

    pub fn get_frpc_info(&self) -> Result<Option<FrpBinaryInfo>, String> {
        let path = self.frpc_binary_path();
        if !path.exists() {
            return Ok(None);
        }

        let (os, arch) = detect_platform();
        Ok(Some(FrpBinaryInfo {
            version: "0.61.1".to_string(),
            path: path.to_string_lossy().to_string(),
            arch: format!("{}_{}", os, arch),
        }))
    }

    pub fn save_config(&self, config: FrpConfig) -> Result<(), String> {
        let mut configs = self.configs.lock().map_err(|e| e.to_string())?;
        configs.insert(config.server_id.clone(), config);
        save_configs(&configs)
    }

    pub fn get_config(&self, server_id: &str) -> Result<Option<FrpConfig>, String> {
        let configs = self.configs.lock().map_err(|e| e.to_string())?;
        Ok(configs.get(server_id).cloned())
    }

    pub fn delete_config(&self, server_id: &str) -> Result<(), String> {
        self.stop_frpc(server_id).ok();
        let mut configs = self.configs.lock().map_err(|e| e.to_string())?;
        configs.remove(server_id);
        save_configs(&configs)
    }

    pub fn start_frpc(&self, server_id: &str) -> Result<(), String> {
        let binary_path = self.frpc_binary_path();
        if !binary_path.exists() {
            return Err("frpc binary not found. Please download it first.".to_string());
        }

        let config = {
            let configs = self.configs.lock().map_err(|e| e.to_string())?;
            configs
                .get(server_id)
                .ok_or_else(|| "FRP config not found for this server".to_string())?
                .clone()
        };

        // Generate config file
        let config_content = generate_frpc_toml(&config);
        let config_path = self.frpc_dir().join(format!("{}.toml", server_id));
        std::fs::write(&config_path, &config_content)
            .map_err(|e| format!("Failed to write frpc config: {}", e))?;

        // Start frpc process
        let child = std::process::Command::new(&binary_path)
            .arg("-c")
            .arg(&config_path)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start frpc: {}", e))?;

        let pid = child.id();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        {
            let mut processes = self.processes.lock().map_err(|e| e.to_string())?;
            processes.insert(server_id.to_string(), child);
        }

        let public_address =
            format!("{}:{}", config.frp_server.server_addr, config.tunnel.remote_port);

        {
            let mut statuses = self.statuses.lock().map_err(|e| e.to_string())?;
            statuses.insert(
                server_id.to_string(),
                FrpStatus {
                    server_id: server_id.to_string(),
                    running: true,
                    pid: Some(pid),
                    public_address: Some(public_address),
                    error: None,
                    started_at: Some(now),
                },
            );
        }

        Ok(())
    }

    pub fn stop_frpc(&self, server_id: &str) -> Result<(), String> {
        let mut processes = self.processes.lock().map_err(|e| e.to_string())?;
        if let Some(mut child) = processes.remove(server_id) {
            let _ = child.kill();
            let _ = child.wait();
        }

        let mut statuses = self.statuses.lock().map_err(|e| e.to_string())?;
        statuses.insert(
            server_id.to_string(),
            FrpStatus {
                server_id: server_id.to_string(),
                running: false,
                pid: None,
                public_address: None,
                error: None,
                started_at: None,
            },
        );

        Ok(())
    }

    pub fn get_status(&self, server_id: &str) -> Result<FrpStatus, String> {
        // Check if process is still running
        let mut running = false;
        {
            let mut processes = self.processes.lock().map_err(|e| e.to_string())?;
            if let Some(child) = processes.get_mut(server_id) {
                match child.try_wait() {
                    Ok(Some(_status)) => {
                        // Process exited
                        processes.remove(server_id);
                    }
                    Ok(None) => {
                        running = true;
                    }
                    Err(_) => {
                        processes.remove(server_id);
                    }
                }
            }
        }

        if !running {
            let mut statuses = self.statuses.lock().map_err(|e| e.to_string())?;
            if let Some(status) = statuses.get_mut(server_id) {
                if status.running {
                    status.running = false;
                    status.pid = None;
                    status.error = Some("Process exited unexpectedly".to_string());
                }
            }
        }

        let statuses = self.statuses.lock().map_err(|e| e.to_string())?;
        Ok(statuses.get(server_id).cloned().unwrap_or(FrpStatus {
            server_id: server_id.to_string(),
            running: false,
            pid: None,
            public_address: None,
            error: None,
            started_at: None,
        }))
    }

    pub fn stop_all(&self) {
        let mut processes = match self.processes.lock() {
            Ok(p) => p,
            Err(_) => return,
        };
        for (_id, mut child) in processes.drain() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn detect_platform() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        "linux"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86") {
        "386"
    } else {
        "amd64"
    };

    (os, arch)
}

fn generate_frpc_toml(config: &FrpConfig) -> String {
    let mut content = String::new();
    content.push_str(&format!(
        "serverAddr = \"{}\"\nserverPort = {}\n",
        config.frp_server.server_addr, config.frp_server.server_port
    ));

    if !config.frp_server.token.is_empty() {
        content.push_str(&format!(
            "\n[auth]\nmethod = \"token\"\ntoken = \"{}\"\n",
            config.frp_server.token
        ));
    }

    content.push_str(&format!(
        "\n[[proxies]]\nname = \"minecraft-{}\"\ntype = \"{}\"\nlocalIP = \"127.0.0.1\"\nlocalPort = {}\nremotePort = {}\n",
        config.server_id,
        config.tunnel.tunnel_type,
        config.tunnel.local_port,
        config.tunnel.remote_port
    ));

    if let Some(ref domains) = config.tunnel.custom_domains {
        if !domains.is_empty() {
            content.push_str(&format!("customDomains = [\"{}\"]\n", domains));
        }
    }

    content
}

fn configs_file_path() -> PathBuf {
    get_app_data_dir().join(FRP_CONFIGS_FILE)
}

fn load_configs() -> HashMap<String, FrpConfig> {
    let path = configs_file_path();
    if !path.exists() {
        return HashMap::new();
    }
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => HashMap::new(),
    }
}

fn save_configs(configs: &HashMap<String, FrpConfig>) -> Result<(), String> {
    let path = configs_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    let content =
        serde_json::to_string_pretty(configs).map_err(|e| format!("Failed to serialize: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("Failed to write: {}", e))
}
