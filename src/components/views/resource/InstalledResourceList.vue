<script setup lang="ts">
import { onMounted } from "vue";
import { useResourceStore } from "@stores/resourceStore";
import { i18n } from "@language";
import SLButton from "@components/common/SLButton.vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import { Trash2, RefreshCw, Package } from "lucide-vue-next";

const props = defineProps<{
  serverId: string;
}>();

const resourceStore = useResourceStore();

onMounted(async () => {
  await resourceStore.loadInstalled(props.serverId);
});

function formatDate(timestamp: number): string {
  return new Date(timestamp * 1000).toLocaleDateString();
}

async function handleUninstall(resourceId: string) {
  await resourceStore.uninstall(props.serverId, resourceId);
}

async function handleCheckUpdates() {
  await resourceStore.checkUpdates(props.serverId);
}
</script>

<template>
  <div class="installed-list">
    <div class="list-header">
      <SLButton size="small" @click="handleCheckUpdates">
        <RefreshCw :size="14" />
        {{ i18n.t("resource.check_updates") }}
      </SLButton>
    </div>

    <div v-if="resourceStore.installedResources.length === 0" class="empty-state">
      <Package :size="48" :stroke-width="1" />
      <p>{{ i18n.t("resource.no_results") }}</p>
    </div>

    <div v-else class="list">
      <div
        v-for="resource in resourceStore.installedResources"
        :key="resource.id"
        class="resource-item"
      >
        <div class="resource-info">
          <div class="resource-name">
            {{ resource.name }}
            <span class="source-badge" :class="resource.source.toLowerCase()">
              {{ resource.source }}
            </span>
          </div>
          <div class="resource-meta">
            <span>v{{ resource.version_number }}</span>
            <span>{{ resource.file_name }}</span>
            <span>{{ formatDate(resource.installed_at) }}</span>
          </div>
        </div>
        <div class="resource-actions">
          <SLButton size="small" type="danger" @click="handleUninstall(resource.id)">
            <Trash2 :size="14" />
            {{ i18n.t("resource.uninstall") }}
          </SLButton>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.installed-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.list-header {
  display: flex;
  justify-content: flex-end;
}

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

.resource-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.resource-name {
  font-weight: 500;
  margin-bottom: 4px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.source-badge {
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 3px;
  font-weight: 500;
}

.source-badge.modrinth {
  background: #1bd96a20;
  color: #1bd96a;
}

.source-badge.curseforge {
  background: #f1643020;
  color: #f16430;
}

.resource-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--text-secondary);
}

.resource-actions {
  display: flex;
  gap: 8px;
}
</style>
