<script setup lang="ts">
import { computed } from "vue";
import { useBackupStore } from "@stores/backupStore";
import { i18n } from "@language";
import SLButton from "@components/common/SLButton.vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import type { BackupMeta } from "@api/backup";
import { Trash2, RotateCcw, FileArchive } from "lucide-vue-next";

const props = defineProps<{
  serverId: string;
  backups: BackupMeta[];
  loading: boolean;
}>();

const backupStore = useBackupStore();

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  if (bytes < 1024 * 1024 * 1024)
    return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  return (bytes / (1024 * 1024 * 1024)).toFixed(2) + " GB";
}

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleString();
}

async function handleRestore(backupId: string) {
  if (!confirm(i18n.t("backup.confirm_restore"))) return;
  try {
    await backupStore.restoreBackup(props.serverId, backupId);
  } catch {
    // error handled by store
  }
}

async function handleDelete(backupId: string) {
  if (!confirm(i18n.t("backup.confirm_delete"))) return;
  try {
    await backupStore.deleteBackup(props.serverId, backupId);
  } catch {
    // error handled by store
  }
}
</script>

<template>
  <div class="backup-list">
    <div v-if="loading" class="loading-state">
      <SLSpinner />
    </div>

    <div v-else-if="backups.length === 0" class="empty-state">
      <FileArchive :size="48" :stroke-width="1" />
      <p>{{ i18n.t("backup.no_backups") }}</p>
    </div>

    <div v-else class="list">
      <div v-for="backup in backups" :key="backup.id" class="backup-item">
        <div class="backup-info">
          <div class="backup-name">{{ backup.name }}</div>
          <div class="backup-meta">
            <span class="badge" :class="backup.backup_type.toLowerCase()">
              {{ i18n.t(`backup.${backup.backup_type.toLowerCase()}`) }}
            </span>
            <span>{{ formatDate(backup.created_at) }}</span>
            <span>{{ formatSize(backup.size_bytes) }}</span>
            <span>{{ backup.file_count }} {{ i18n.t("backup.file_count") }}</span>
          </div>
        </div>
        <div class="backup-actions">
          <SLButton size="small" @click="handleRestore(backup.id)">
            <RotateCcw :size="14" />
            {{ i18n.t("backup.restore") }}
          </SLButton>
          <SLButton size="small" type="danger" @click="handleDelete(backup.id)">
            <Trash2 :size="14" />
          </SLButton>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.backup-list {
  flex: 1;
  overflow: auto;
}

.loading-state,
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-secondary);
  gap: 12px;
}

.list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.backup-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.backup-name {
  font-weight: 500;
  margin-bottom: 4px;
}

.backup-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--text-secondary);
}

.badge {
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

.badge.full {
  background: var(--color-primary-light, #e3f2fd);
  color: var(--color-primary, #1976d2);
}

.badge.incremental {
  background: var(--color-success-light, #e8f5e9);
  color: var(--color-success, #388e3c);
}

.backup-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}
</style>
