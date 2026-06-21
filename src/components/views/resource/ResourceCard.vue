<script setup lang="ts">
import { ref } from "vue";
import { useResourceStore } from "@stores/resourceStore";
import { i18n } from "@language";
import SLButton from "@components/common/SLButton.vue";
import type { ResourceSearchResult } from "@api/resource";
import { Download, Package } from "lucide-vue-next";

const props = defineProps<{
  resource: ResourceSearchResult;
  serverId: string;
}>();

const resourceStore = useResourceStore();
const installing = ref(false);

function formatDownloads(n: number): string {
  if (n >= 1000000) return (n / 1000000).toFixed(1) + "M";
  if (n >= 1000) return (n / 1000).toFixed(1) + "K";
  return String(n);
}

async function handleInstall() {
  installing.value = true;
  try {
    // Get versions and install the first one
    const { resourceApi } = await import("@api/resource");
    const source = props.resource.source === "Modrinth" ? "modrinth" : "curseforge";
    const versions = await resourceApi.getVersions(source, props.resource.id, null, null);
    if (versions.length > 0) {
      await resourceStore.install(props.serverId, source, props.resource.id, versions[0].id);
    }
  } catch {
    // error handled by store
  } finally {
    installing.value = false;
  }
}
</script>

<template>
  <div class="resource-card">
    <div class="card-icon">
      <img v-if="resource.icon_url" :src="resource.icon_url" alt="" />
      <Package v-else :size="32" :stroke-width="1" />
    </div>
    <div class="card-content">
      <div class="card-header">
        <span class="card-name">{{ resource.name }}</span>
        <span class="source-badge" :class="resource.source.toLowerCase()">
          {{ resource.source }}
        </span>
      </div>
      <div class="card-summary">{{ resource.summary }}</div>
      <div class="card-meta">
        <span>{{ resource.author }}</span>
        <span><Download :size="12" /> {{ formatDownloads(resource.downloads) }}</span>
      </div>
    </div>
    <div class="card-actions">
      <SLButton size="small" type="primary" :loading="installing" @click="handleInstall">
        {{ i18n.t("resource.install") }}
      </SLButton>
    </div>
  </div>
</template>

<style scoped>
.resource-card {
  display: flex;
  gap: 12px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
}

.card-icon {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  overflow: hidden;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
}

.card-icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.card-content {
  flex: 1;
  min-width: 0;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.card-name {
  font-weight: 500;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.source-badge {
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 3px;
  font-weight: 500;
  flex-shrink: 0;
}

.source-badge.modrinth {
  background: #1bd96a20;
  color: #1bd96a;
}

.source-badge.curseforge {
  background: #f1643020;
  color: #f16430;
}

.card-summary {
  font-size: 12px;
  color: var(--text-secondary);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  margin-bottom: 6px;
}

.card-meta {
  display: flex;
  gap: 12px;
  font-size: 11px;
  color: var(--text-tertiary);
}

.card-meta span {
  display: flex;
  align-items: center;
  gap: 3px;
}

.card-actions {
  display: flex;
  align-items: flex-start;
}
</style>
