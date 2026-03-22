<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { useServerStore } from "@stores/serverStore";
import { useSchedulerStore } from "@stores/schedulerStore";
import { i18n } from "@language";
import SLButton from "@components/common/SLButton.vue";
import SLServerSelector from "@components/common/SLServerSelector.vue";
import TaskList from "@components/views/scheduler/TaskList.vue";
import CreateTaskModal from "@components/views/scheduler/CreateTaskModal.vue";

const serverStore = useServerStore();
const schedulerStore = useSchedulerStore();

const showCreateModal = ref(false);

const selectedServerId = computed(() => serverStore.currentServerId || "");

const serverOptions = computed(() =>
  serverStore.servers.map((s) => ({ label: s.name, value: s.id })),
);

function onServerChange(id: string) {
  serverStore.setCurrentServer(id);
}

onMounted(async () => {
  await serverStore.refreshList();
  if (!serverStore.currentServerId && serverStore.servers.length > 0) {
    serverStore.setCurrentServer(serverStore.servers[0].id);
  }
  if (selectedServerId.value) {
    await schedulerStore.refreshTasks(selectedServerId.value);
  }
});

watch(selectedServerId, async (sid) => {
  if (sid) {
    await schedulerStore.refreshTasks(sid);
  }
});

async function handleCreated() {
  showCreateModal.value = false;
  if (selectedServerId.value) {
    await schedulerStore.refreshTasks(selectedServerId.value);
  }
}
</script>

<template>
  <div class="view-container">
    <div class="view-header">
      <div class="header-left">
        <h2>{{ i18n.t("scheduler.title") }}</h2>
        <SLServerSelector
          :model-value="selectedServerId"
          :options="serverOptions"
          @update:model-value="onServerChange"
        />
      </div>
      <div class="header-actions">
        <SLButton
          type="primary"
          :disabled="!selectedServerId"
          @click="showCreateModal = true"
        >
          {{ i18n.t("scheduler.create") }}
        </SLButton>
      </div>
    </div>

    <TaskList
      v-if="selectedServerId"
      :server-id="selectedServerId"
      :tasks="schedulerStore.tasks"
      :loading="schedulerStore.loading"
    />

    <div v-else class="empty-state">
      {{ i18n.t("common.select_server") }}
    </div>

    <CreateTaskModal
      v-if="showCreateModal"
      :server-id="selectedServerId"
      @close="showCreateModal = false"
      @created="handleCreated"
    />
  </div>
</template>

<style scoped>
.view-container {
  padding: 20px;
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.view-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-left h2 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--text-secondary);
}
</style>
