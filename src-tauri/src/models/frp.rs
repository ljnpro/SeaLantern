use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpConfig {
    pub id: String,
    pub server_id: String,
    pub enabled: bool,
    pub frp_server: FrpServerConfig,
    pub tunnel: FrpTunnelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpServerConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub token: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpTunnelConfig {
    pub tunnel_type: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub custom_domains: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpStatus {
    pub server_id: String,
    pub running: bool,
    pub pid: Option<u32>,
    pub public_address: Option<String>,
    pub error: Option<String>,
    pub started_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrpBinaryInfo {
    pub version: String,
    pub path: String,
    pub arch: String,
}
