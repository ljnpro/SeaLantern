import { tauriInvoke } from "@api/tauri";

export interface FrpConfig {
  id: string;
  server_id: string;
  enabled: boolean;
  frp_server: FrpServerConfig;
  tunnel: FrpTunnelConfig;
}

export interface FrpServerConfig {
  server_addr: string;
  server_port: number;
  token: string;
  provider: string;
}

export interface FrpTunnelConfig {
  tunnel_type: string;
  local_port: number;
  remote_port: number;
  custom_domains: string | null;
}

export interface FrpStatus {
  server_id: string;
  running: boolean;
  pid: number | null;
  public_address: string | null;
  error: string | null;
  started_at: number | null;
}

export interface FrpBinaryInfo {
  version: string;
  path: string;
  arch: string;
}

export const frpApi = {
  async downloadFrpc(): Promise<FrpBinaryInfo> {
    return tauriInvoke("download_frpc");
  },

  async getFrpcInfo(): Promise<FrpBinaryInfo | null> {
    return tauriInvoke("get_frpc_info");
  },

  async saveConfig(config: FrpConfig): Promise<void> {
    return tauriInvoke("save_frp_config", { config });
  },

  async getConfig(serverId: string): Promise<FrpConfig | null> {
    return tauriInvoke("get_frp_config", { serverId });
  },

  async deleteConfig(serverId: string): Promise<void> {
    return tauriInvoke("delete_frp_config", { serverId });
  },

  async start(serverId: string): Promise<void> {
    return tauriInvoke("start_frpc", { serverId });
  },

  async stop(serverId: string): Promise<void> {
    return tauriInvoke("stop_frpc", { serverId });
  },

  async getStatus(serverId: string): Promise<FrpStatus> {
    return tauriInvoke("get_frp_status", { serverId });
  },
};
