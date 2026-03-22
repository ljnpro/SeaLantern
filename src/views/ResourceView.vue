<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { useServerStore } from "@stores/serverStore";
import { useResourceStore } from "@stores/resourceStore";
import { i18n } from "@language";
import SLTabBar from "@components/common/SLTabBar.vue";
import SLServerSelector from "@components/common/SLServerSelector.vue";
import ResourceBrowser from "@components/views/resource/ResourceBrowser.vue";
import InstalledResourceList from "@components/views/resource/InstalledResourceList.vue";

const serverStore = useServerStore();
const resourceStore = useResourceStore();

const activeTab = ref<"browse" | "installed">("browse");

const selectedServerId = computed(() => serverStore.currentServerId || "");

const serverOptions = computed(() =>
  serverStore.servers.map((s) => ({ label: s.name, value: s.id })),
);

const tabs = computed(() => [
  { key: "browse", label: i18n.t("resource.browse") },
  { key: "installed", label: i18n.t("resource.installed") },
]);

function onServerChange(id: string) {
  serverStore.setCurrentServer(id);
}

onMounted(async () => {
  await serverStore.refreshList();
  if (!serverStore.currentServerId && serverStore.servers.length > 0) {
    serverStore.setCurrentServer(serverStore.servers[0].id);
  }
  if (selectedServerId.value) {
    await resourceStore.loadInstalled(selectedServerId.value);
  }
});

watch(selectedServerId, async (sid) => {
  if (sid) {
    await resourceStore.loadInstalled(sid);
  }
});
</script>

<template>
  <div class="view-container">
    <div class="view-header">
      <div class="header-left">
        <h2>{{ i18n.t("resource.title") }}</h2>
        <SLServerSelector
          :model-value="selectedServerId"
          :options="serverOptions"
          @update:model-value="onServerChange"
        />
      </div>
    </div>

    <SLTabBar
      v-model="activeTab"
      :tabs="tabs"
    />

    <div v-if="selectedServerId" class="tab-content">
      <ResourceBrowser
        v-if="activeTab === 'browse'"
        :server-id="selectedServerId"
      />
      <InstalledResourceList
        v-else
        :server-id="selectedServerId"
      />
    </div>

    <div v-else class="empty-state">
      {{ i18n.t("common.select_server") }}
    </div>
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

.tab-content {
  flex: 1;
  overflow: auto;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--text-secondary);
}
</style>
