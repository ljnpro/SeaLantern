<script setup lang="ts">
import { ref } from "vue";
import { useResourceStore } from "@stores/resourceStore";
import { i18n } from "@language";
import SLInput from "@components/common/SLInput.vue";
import SLButton from "@components/common/SLButton.vue";
import SLSpinner from "@components/common/SLSpinner.vue";
import ResourceCard from "@components/views/resource/ResourceCard.vue";
import { Search } from "lucide-vue-next";

const props = defineProps<{
  serverId: string;
}>();

const resourceStore = useResourceStore();
const searchQuery = ref("");
const sourceFilter = ref<string | null>(null);

async function handleSearch() {
  if (!searchQuery.value.trim()) return;
  await resourceStore.search(searchQuery.value, null, null, null, sourceFilter.value);
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Enter") handleSearch();
}
</script>

<template>
  <div class="resource-browser">
    <div class="search-bar">
      <SLInput
        v-model="searchQuery"
        :placeholder="i18n.t('resource.search_placeholder')"
        @keydown="handleKeydown"
      />
      <div class="source-filters">
        <SLButton
          :type="sourceFilter === null ? 'primary' : 'default'"
          size="small"
          @click="sourceFilter = null"
        >
          {{ i18n.t("resource.source_all") }}
        </SLButton>
        <SLButton
          :type="sourceFilter === 'modrinth' ? 'primary' : 'default'"
          size="small"
          @click="sourceFilter = 'modrinth'"
        >
          {{ i18n.t("resource.modrinth") }}
        </SLButton>
        <SLButton
          :type="sourceFilter === 'curseforge' ? 'primary' : 'default'"
          size="small"
          @click="sourceFilter = 'curseforge'"
        >
          {{ i18n.t("resource.curseforge") }}
        </SLButton>
      </div>
      <SLButton type="primary" @click="handleSearch">
        <Search :size="16" />
      </SLButton>
    </div>

    <div v-if="resourceStore.searchLoading" class="loading-state">
      <SLSpinner />
    </div>

    <div v-else-if="resourceStore.searchResults.length === 0" class="empty-state">
      <p>{{ i18n.t("resource.no_results") }}</p>
    </div>

    <div v-else class="results-grid">
      <ResourceCard
        v-for="result in resourceStore.searchResults"
        :key="result.id + result.source"
        :resource="result"
        :server-id="serverId"
      />
    </div>
  </div>
</template>

<style scoped>
.resource-browser {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.search-bar {
  display: flex;
  gap: 8px;
  align-items: center;
}

.source-filters {
  display: flex;
  gap: 4px;
}

.loading-state,
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: var(--text-secondary);
}

.results-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 12px;
}
</style>
