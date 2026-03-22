import { defineStore } from "pinia";
import { ref } from "vue";
import { frpApi } from "@api/frp";
import type { FrpConfig, FrpStatus, FrpBinaryInfo } from "@api/frp";
import { useLoading } from "@composables/useAsync";

export const useFrpStore = defineStore("frp", () => {
  const config = ref<FrpConfig | null>(null);
  const status = ref<FrpStatus | null>(null);
  const binaryInfo = ref<FrpBinaryInfo | null>(null);
  const { loading, withLoading } = useLoading(false);
  const error = ref<string | null>(null);

  async function loadConfig(serverId: string) {
    error.value = null;
    try {
      config.value = await frpApi.getConfig(serverId);
    } catch (e) {
      error.value = String(e);
    }
  }

  async function saveConfig(frpConfig: FrpConfig) {
    error.value = null;
    try {
      await frpApi.saveConfig(frpConfig);
      config.value = frpConfig;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function refreshStatus(serverId: string) {
    try {
      status.value = await frpApi.getStatus(serverId);
    } catch (e) {
      error.value = String(e);
    }
  }

  async function start(serverId: string) {
    error.value = null;
    try {
      await withLoading(() => frpApi.start(serverId));
      await refreshStatus(serverId);
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function stop(serverId: string) {
    error.value = null;
    try {
      await frpApi.stop(serverId);
      await refreshStatus(serverId);
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function checkBinary() {
    try {
      binaryInfo.value = await frpApi.getFrpcInfo();
    } catch (e) {
      error.value = String(e);
    }
  }

  async function downloadBinary() {
    error.value = null;
    try {
      binaryInfo.value = await withLoading(() => frpApi.downloadFrpc());
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  return {
    config,
    status,
    binaryInfo,
    loading,
    error,
    loadConfig,
    saveConfig,
    refreshStatus,
    start,
    stop,
    checkBinary,
    downloadBinary,
  };
});
