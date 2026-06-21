<script setup lang="ts">
import { ref, computed } from "vue";
import { useSchedulerStore } from "@stores/schedulerStore";
import { i18n } from "@language";
import SLModal from "@components/common/SLModal.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLButton from "@components/common/SLButton.vue";
import type { TaskType, TaskSchedule } from "@api/scheduler";

const props = defineProps<{
  serverId: string;
}>();

const emit = defineEmits<{
  close: [];
  created: [];
}>();

const schedulerStore = useSchedulerStore();
const name = ref("");
const taskTypeKey = ref("RestartServer");
const scheduleType = ref("interval");
const command = ref("");
const intervalSeconds = ref(3600);
const dailyHour = ref(4);
const dailyMinute = ref(0);
const creating = ref(false);

const taskTypeOptions = computed(() => [
  { label: i18n.t("scheduler.restart_server"), value: "RestartServer" },
  { label: i18n.t("scheduler.run_backup"), value: "RunBackup" },
  { label: i18n.t("scheduler.execute_command"), value: "ExecuteCommand" },
]);

const scheduleTypeOptions = computed(() => [
  { label: i18n.t("scheduler.interval"), value: "interval" },
  { label: i18n.t("scheduler.daily"), value: "daily" },
]);

function buildTaskType(): TaskType {
  switch (taskTypeKey.value) {
    case "RunBackup":
      return { RunBackup: { incremental: false, include_paths: ["world"] } };
    case "ExecuteCommand":
      return { ExecuteCommand: { command: command.value } };
    default:
      return "RestartServer";
  }
}

function buildSchedule(): TaskSchedule {
  if (scheduleType.value === "daily") {
    return { Daily: { hour: dailyHour.value, minute: dailyMinute.value } };
  }
  return { Interval: { seconds: intervalSeconds.value } };
}

async function handleCreate() {
  if (!name.value.trim()) return;
  creating.value = true;
  try {
    await schedulerStore.createTask(props.serverId, name.value, buildTaskType(), buildSchedule());
    emit("created");
  } catch {
    // error handled by store
  } finally {
    creating.value = false;
  }
}
</script>

<template>
  <SLModal :title="i18n.t('scheduler.create')" @close="emit('close')">
    <div class="form">
      <div class="form-group">
        <label>{{ i18n.t("scheduler.name") }}</label>
        <SLInput v-model="name" :placeholder="i18n.t('scheduler.name')" />
      </div>

      <div class="form-group">
        <label>{{ i18n.t("scheduler.type") }}</label>
        <SLSelect
          :model-value="taskTypeKey"
          :options="taskTypeOptions"
          @update:model-value="taskTypeKey = $event"
        />
      </div>

      <div v-if="taskTypeKey === 'ExecuteCommand'" class="form-group">
        <label>{{ i18n.t("scheduler.execute_command") }}</label>
        <SLInput v-model="command" placeholder="say Hello" />
      </div>

      <div class="form-group">
        <label>{{ i18n.t("scheduler.schedule") }}</label>
        <SLSelect
          :model-value="scheduleType"
          :options="scheduleTypeOptions"
          @update:model-value="scheduleType = $event"
        />
      </div>

      <div v-if="scheduleType === 'interval'" class="form-group">
        <label>{{ i18n.t("scheduler.every_n_seconds", { n: "" }) }}</label>
        <SLInput v-model.number="intervalSeconds" type="number" />
      </div>

      <div v-if="scheduleType === 'daily'" class="form-group row">
        <label>{{ i18n.t("scheduler.at_time", { time: "" }) }}</label>
        <div class="time-inputs">
          <SLInput v-model.number="dailyHour" type="number" style="width: 60px" />
          <span>:</span>
          <SLInput v-model.number="dailyMinute" type="number" style="width: 60px" />
        </div>
      </div>
    </div>

    <template #footer>
      <SLButton @click="emit('close')">{{ i18n.t("common.cancel") }}</SLButton>
      <SLButton type="primary" :loading="creating" @click="handleCreate">
        {{ i18n.t("scheduler.create") }}
      </SLButton>
    </template>
  </SLModal>
</template>

<style scoped>
.form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group.row {
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
}

.form-group label {
  font-size: 13px;
  font-weight: 500;
}

.time-inputs {
  display: flex;
  align-items: center;
  gap: 4px;
}
</style>
