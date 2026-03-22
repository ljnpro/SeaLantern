<script setup lang="ts">
import { useSchedulerStore } from "@stores/schedulerStore";
import { i18n } from "@language";
import SLButton from "@components/common/SLButton.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import type { ScheduledTask } from "@api/scheduler";
import { Trash2, Play, Clock } from "lucide-vue-next";

const props = defineProps<{
  serverId: string;
  tasks: ScheduledTask[];
  loading: boolean;
}>();

const schedulerStore = useSchedulerStore();

function getTaskTypeLabel(task: ScheduledTask): string {
  if (task.task_type === "RestartServer") return i18n.t("scheduler.restart_server");
  if (typeof task.task_type === "object") {
    if ("RunBackup" in task.task_type) return i18n.t("scheduler.run_backup");
    if ("ExecuteCommand" in task.task_type) return i18n.t("scheduler.execute_command");
  }
  return "";
}

function getScheduleLabel(task: ScheduledTask): string {
  const schedule = task.schedule;
  if ("Interval" in schedule) {
    const secs = schedule.Interval.seconds;
    if (secs >= 3600) return i18n.t("scheduler.every_n_hours", { n: Math.floor(secs / 3600) });
    if (secs >= 60) return i18n.t("scheduler.every_n_minutes", { n: Math.floor(secs / 60) });
    return i18n.t("scheduler.every_n_seconds", { n: secs });
  }
  if ("Daily" in schedule) {
    const h = String(schedule.Daily.hour).padStart(2, "0");
    const m = String(schedule.Daily.minute).padStart(2, "0");
    return i18n.t("scheduler.at_time", { time: `${h}:${m}` });
  }
  return "";
}

function formatDate(timestamp: number | null): string {
  if (!timestamp) return "-";
  return new Date(timestamp * 1000).toLocaleString();
}

async function handleToggle(taskId: string, enabled: boolean) {
  await schedulerStore.toggleTask(taskId, enabled);
}

async function handleRunNow(taskId: string) {
  await schedulerStore.runTaskNow(taskId);
}

async function handleDelete(taskId: string) {
  if (!confirm(i18n.t("scheduler.confirm_delete"))) return;
  await schedulerStore.deleteTask(taskId);
}
</script>

<template>
  <div class="task-list">
    <div v-if="loading" class="loading-state">
      <SLSpinner />
    </div>

    <div v-else-if="tasks.length === 0" class="empty-state">
      <Clock :size="48" :stroke-width="1" />
      <p>{{ i18n.t("scheduler.no_tasks") }}</p>
    </div>

    <div v-else class="list">
      <div v-for="task in tasks" :key="task.id" class="task-item">
        <div class="task-left">
          <SLSwitch
            :model-value="task.enabled"
            @update:model-value="handleToggle(task.id, $event)"
          />
        </div>
        <div class="task-info">
          <div class="task-name">{{ task.name }}</div>
          <div class="task-meta">
            <span class="task-type">{{ getTaskTypeLabel(task) }}</span>
            <span>{{ getScheduleLabel(task) }}</span>
            <span v-if="task.last_run_at">
              {{ i18n.t("scheduler.last_run") }}: {{ formatDate(task.last_run_at) }}
            </span>
            <span
              v-if="task.last_run_result"
              :class="{ success: task.last_run_result === 'success', failed: task.last_run_result !== 'success' }"
            >
              {{ task.last_run_result === "success" ? i18n.t("scheduler.result_success") : i18n.t("scheduler.result_failed") }}
            </span>
          </div>
        </div>
        <div class="task-actions">
          <SLButton size="small" @click="handleRunNow(task.id)">
            <Play :size="14" />
            {{ i18n.t("scheduler.run_now") }}
          </SLButton>
          <SLButton size="small" type="danger" @click="handleDelete(task.id)">
            <Trash2 :size="14" />
          </SLButton>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.task-list {
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

.task-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.task-info {
  flex: 1;
}

.task-name {
  font-weight: 500;
  margin-bottom: 4px;
}

.task-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--text-secondary);
}

.task-type {
  font-weight: 500;
  color: var(--color-primary);
}

.success {
  color: var(--color-success, #388e3c);
}

.failed {
  color: var(--color-error, #d32f2f);
}

.task-actions {
  display: flex;
  gap: 8px;
}
</style>
