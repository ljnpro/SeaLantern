<script setup lang="ts">
import { useFrpStore } from "@stores/frpStore";
import { i18n } from "@language";
import SLButton from "@components/common/SLButton.vue";
import { Play, Square, Copy, Wifi, WifiOff } from "lucide-vue-next";

const props = defineProps<{
  serverId: string;
}>();

const frpStore = useFrpStore();

async function handleStart() {
  await frpStore.start(props.serverId);
}

async function handleStop() {
  await frpStore.stop(props.serverId);
}

function copyAddress() {
  if (frpStore.status?.public_address) {
    navigator.clipboard.writeText(frpStore.status.public_address);
  }
}
</script>

<template>
  <div class="frp-status-panel">
    <div class="status-indicator" :class="{ running: frpStore.status?.running }">
      <Wifi v-if="frpStore.status?.running" :size="20" />
      <WifiOff v-else :size="20" />
      <span>
        {{
          frpStore.status?.running
            ? i18n.t("frp.status_running")
            : i18n.t("frp.status_stopped")
        }}
      </span>
    </div>

    <div v-if="frpStore.status?.public_address" class="public-address">
      <span class="label">{{ i18n.t("frp.public_address") }}:</span>
      <code>{{ frpStore.status.public_address }}</code>
      <SLButton size="small" @click="copyAddress">
        <Copy :size="12" />
      </SLButton>
    </div>

    <div v-if="frpStore.status?.error" class="error">
      {{ frpStore.status.error }}
    </div>

    <div class="status-actions">
      <SLButton
        v-if="!frpStore.status?.running"
        type="primary"
        :loading="frpStore.loading"
        @click="handleStart"
      >
        <Play :size="14" />
        {{ i18n.t("frp.start") }}
      </SLButton>
      <SLButton
        v-else
        type="danger"
        @click="handleStop"
      >
        <Square :size="14" />
        {{ i18n.t("frp.stop") }}
      </SLButton>
    </div>
  </div>
</template>

<style scoped>
.frp-status-panel {
  background: var(--bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  color: var(--text-secondary);
}

.status-indicator.running {
  color: var(--color-success, #388e3c);
}

.public-address {
  display: flex;
  align-items: center;
  gap: 8px;
}

.public-address .label {
  font-size: 13px;
  color: var(--text-secondary);
}

.public-address code {
  background: var(--bg-tertiary);
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 13px;
}

.error {
  color: var(--color-error, #d32f2f);
  font-size: 13px;
}

.status-actions {
  margin-left: auto;
}
</style>
