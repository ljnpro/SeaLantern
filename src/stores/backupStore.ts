import { defineStore } from "pinia";
import { ref } from "vue";
import { backupApi } from "@api/backup";
import type { BackupMeta, BackupManifest } from "@api/backup";
import { useLoading } from "@composables/useAsync";

export const useBackupStore = defineStore("backup", () => {
  const backups = ref<BackupMeta[]>([]);
  const currentDetail = ref<BackupManifest | null>(null);
  const { loading, withLoading } = useLoading(false);
  const error = ref<string | null>(null);

  async function refreshBackups(serverId: string) {
    error.value = null;
    try {
      backups.value = await withLoading(() => backupApi.listBackups(serverId));
    } catch (e) {
      error.value = String(e);
    }
  }

  async function createBackup(
    serverId: string,
    name: string | null,
    incremental: boolean,
    includePaths: string[],
  ) {
    error.value = null;
    try {
      const meta = await withLoading(() =>
        backupApi.createBackup(serverId, name, incremental, includePaths),
      );
      backups.value.push(meta);
      return meta;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function restoreBackup(serverId: string, backupId: string) {
    error.value = null;
    try {
      await withLoading(() => backupApi.restoreBackup(serverId, backupId));
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function deleteBackup(serverId: string, backupId: string) {
    error.value = null;
    try {
      await backupApi.deleteBackup(serverId, backupId);
      backups.value = backups.value.filter((b) => b.id !== backupId);
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function loadDetail(serverId: string, backupId: string) {
    try {
      currentDetail.value = await backupApi.getBackupDetail(serverId, backupId);
    } catch (e) {
      error.value = String(e);
    }
  }

  return {
    backups,
    currentDetail,
    loading,
    error,
    refreshBackups,
    createBackup,
    restoreBackup,
    deleteBackup,
    loadDetail,
  };
});
