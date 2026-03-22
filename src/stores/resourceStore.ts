import { defineStore } from "pinia";
import { ref } from "vue";
import { resourceApi } from "@api/resource";
import type {
  ResourceSearchResult,
  ResourceDetail,
  InstalledResource,
  ResourceUpdateInfo,
} from "@api/resource";
import { useLoading } from "@composables/useAsync";

export const useResourceStore = defineStore("resource", () => {
  const searchResults = ref<ResourceSearchResult[]>([]);
  const installedResources = ref<InstalledResource[]>([]);
  const resourceDetail = ref<ResourceDetail | null>(null);
  const updates = ref<ResourceUpdateInfo[]>([]);
  const { loading: searchLoading, withLoading: withSearchLoading } =
    useLoading(false);
  const { loading: installLoading, withLoading: withInstallLoading } =
    useLoading(false);
  const error = ref<string | null>(null);

  async function search(
    query: string,
    gameVersion: string | null = null,
    loader: string | null = null,
    projectType: string | null = null,
    source: string | null = null,
    offset = 0,
    limit = 20,
  ) {
    error.value = null;
    try {
      searchResults.value = await withSearchLoading(() =>
        resourceApi.search(
          query,
          gameVersion,
          loader,
          projectType,
          source,
          offset,
          limit,
        ),
      );
    } catch (e) {
      error.value = String(e);
    }
  }

  async function loadDetail(source: string, projectId: string) {
    error.value = null;
    try {
      resourceDetail.value = await resourceApi.getDetail(source, projectId);
    } catch (e) {
      error.value = String(e);
    }
  }

  async function loadInstalled(serverId: string) {
    error.value = null;
    try {
      installedResources.value = await resourceApi.getInstalled(serverId);
    } catch (e) {
      error.value = String(e);
    }
  }

  async function install(
    serverId: string,
    source: string,
    projectId: string,
    versionId: string,
  ) {
    error.value = null;
    try {
      const installed = await withInstallLoading(() =>
        resourceApi.install(serverId, source, projectId, versionId),
      );
      installedResources.value.push(installed);
      return installed;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function uninstall(serverId: string, resourceId: string) {
    error.value = null;
    try {
      await resourceApi.uninstall(serverId, resourceId);
      installedResources.value = installedResources.value.filter(
        (r) => r.id !== resourceId,
      );
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function checkUpdates(serverId: string) {
    error.value = null;
    try {
      updates.value = await resourceApi.checkUpdates(serverId);
    } catch (e) {
      error.value = String(e);
    }
  }

  async function updateResource(
    serverId: string,
    resourceId: string,
    versionId: string,
  ) {
    error.value = null;
    try {
      const updated = await withInstallLoading(() =>
        resourceApi.update(serverId, resourceId, versionId),
      );
      const idx = installedResources.value.findIndex(
        (r) => r.id === resourceId,
      );
      if (idx >= 0) installedResources.value[idx] = updated;
      return updated;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  return {
    searchResults,
    installedResources,
    resourceDetail,
    updates,
    searchLoading,
    installLoading,
    error,
    search,
    loadDetail,
    loadInstalled,
    install,
    uninstall,
    checkUpdates,
    updateResource,
  };
});
