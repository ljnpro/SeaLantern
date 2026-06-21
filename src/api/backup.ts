import { tauriInvoke } from "@api/tauri";

export interface BackupMeta {
  id: string;
  server_id: string;
  name: string;
  backup_type: "Full" | "Incremental";
  created_at: number;
  size_bytes: number;
  file_count: number;
  parent_id: string | null;
  paths_included: string[];
  notes: string | null;
}

export interface BackupFileEntry {
  relative_path: string;
  sha256: string;
  size: number;
  modified_at: number;
}

export interface BackupManifest {
  meta: BackupMeta;
  files: BackupFileEntry[];
}

export const backupApi = {
  async createBackup(
    serverId: string,
    name: string | null,
    incremental: boolean,
    includePaths: string[],
  ): Promise<BackupMeta> {
    return tauriInvoke("create_backup", {
      serverId,
      name,
      incremental,
      includePaths,
    });
  },

  async listBackups(serverId: string): Promise<BackupMeta[]> {
    return tauriInvoke("list_backups", { serverId });
  },

  async getBackupDetail(serverId: string, backupId: string): Promise<BackupManifest> {
    return tauriInvoke("get_backup_detail", { serverId, backupId });
  },

  async restoreBackup(serverId: string, backupId: string): Promise<void> {
    return tauriInvoke("restore_backup", { serverId, backupId });
  },

  async deleteBackup(serverId: string, backupId: string): Promise<void> {
    return tauriInvoke("delete_backup", { serverId, backupId });
  },
};
