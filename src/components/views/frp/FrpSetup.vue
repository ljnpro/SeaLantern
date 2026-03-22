<script setup lang="ts">
import { useFrpStore } from "@stores/frpStore";
import { i18n } from "@language";
import SLButton from "@components/common/SLButton.vue";
import { Download } from "lucide-vue-next";

const frpStore = useFrpStore();

async function handleDownload() {
  await frpStore.downloadBinary();
}
</script>

<template>
  <div class="frp-setup">
    <div class="setup-content">
      <Download :size="48" :stroke-width="1" />
      <h3>{{ i18n.t("frp.not_installed") }}</h3>
      <p>{{ i18n.t("frp.download_frpc") }}</p>
      <SLButton type="primary" :loading="frpStore.loading" @click="handleDownload">
        {{ frpStore.loading ? i18n.t("frp.downloading") : i18n.t("frp.download_frpc") }}
      </SLButton>
      <p v-if="frpStore.error" class="error">{{ frpStore.error }}</p>
    </div>
  </div>
</template>

<style scoped>
.frp-setup {
  display: flex;
  align-items: center;
  justify-content: center;
  flex: 1;
}

.setup-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  text-align: center;
  color: var(--text-secondary);
}

.setup-content h3 {
  margin: 0;
  color: var(--text-primary);
}

.error {
  color: var(--color-error, #d32f2f);
  font-size: 13px;
}
</style>
