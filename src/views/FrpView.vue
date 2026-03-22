<script setup lang="ts">
import { onMounted, computed, watch } from "vue";
import { useServerStore } from "@stores/serverStore";
import { useFrpStore } from "@stores/frpStore";
import { i18n } from "@language";
import SLServerSelector from "@components/common/SLServerSelector.vue";
import FrpSetup from "@components/views/frp/FrpSetup.vue";
import FrpConfigForm from "@components/views/frp/FrpConfigForm.vue";
import FrpStatusPanel from "@components/views/frp/FrpStatusPanel.vue";

const serverStore = useServerStore();
const frpStore = useFrpStore();

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
  await frpStore.checkBinary();
  if (selectedServerId.value) {
    await frpStore.loadConfig(selectedServerId.value);
    await frpStore.refreshStatus(selectedServerId.value);
  }
});

watch(selectedServerId, async (sid) => {
  if (sid) {
    await frpStore.loadConfig(sid);
    await frpStore.refreshStatus(sid);
  }
});
</script>

<template>
  <div class="view-container">
    <div class="view-header">
      <div class="header-left">
        <h2>{{ i18n.t("frp.title") }}</h2>
        <SLServerSelector
          :model-value="selectedServerId"
          :options="serverOptions"
          @update:model-value="onServerChange"
        />
      </div>
    </div>

    <template v-if="selectedServerId">
      <FrpSetup v-if="!frpStore.binaryInfo" />
      <template v-else>
        <FrpStatusPanel :server-id="selectedServerId" />
        <FrpConfigForm :server-id="selectedServerId" />
      </template>
    </template>

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

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
  color: var(--text-secondary);
}
</style>
