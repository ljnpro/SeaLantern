<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { useFrpStore } from "@stores/frpStore";
import { useServerStore } from "@stores/serverStore";
import { i18n } from "@language";
import SLInput from "@components/common/SLInput.vue";
import SLSelect from "@components/common/SLSelect.vue";
import SLButton from "@components/common/SLButton.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import type { FrpConfig } from "@api/frp";

const props = defineProps<{
  serverId: string;
}>();

const frpStore = useFrpStore();
const serverStore = useServerStore();

const serverAddr = ref("");
const serverPort = ref(7000);
const token = ref("");
const provider = ref("custom");
const tunnelType = ref("tcp");
const localPort = ref(25565);
const remotePort = ref(25565);
const autoStart = ref(false);
const saving = ref(false);

const providerOptions = [
  { label: i18n.t("frp.custom"), value: "custom" },
  { label: "Sakura Frp", value: "sakura" },
  { label: "OpenFRP", value: "openfrp" },
];

const tunnelTypeOptions = [
  { label: "TCP", value: "tcp" },
  { label: "UDP", value: "udp" },
];

function loadFromConfig(config: FrpConfig | null) {
  if (!config) {
    // Set defaults from server's port
    const server = serverStore.servers.find((s) => s.id === props.serverId);
    if (server) {
      localPort.value = server.port;
      remotePort.value = server.port;
    }
    return;
  }
  serverAddr.value = config.frp_server.server_addr;
  serverPort.value = config.frp_server.server_port;
  token.value = config.frp_server.token;
  provider.value = config.frp_server.provider;
  tunnelType.value = config.tunnel.tunnel_type;
  localPort.value = config.tunnel.local_port;
  remotePort.value = config.tunnel.remote_port;
  autoStart.value = config.enabled;
}

onMounted(() => {
  loadFromConfig(frpStore.config);
});

watch(() => frpStore.config, loadFromConfig);

async function handleSave() {
  saving.value = true;
  try {
    const config: FrpConfig = {
      id: frpStore.config?.id || crypto.randomUUID(),
      server_id: props.serverId,
      enabled: autoStart.value,
      frp_server: {
        server_addr: serverAddr.value,
        server_port: serverPort.value,
        token: token.value,
        provider: provider.value,
      },
      tunnel: {
        tunnel_type: tunnelType.value,
        local_port: localPort.value,
        remote_port: remotePort.value,
        custom_domains: null,
      },
    };
    await frpStore.saveConfig(config);
  } catch {
    // error handled by store
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="frp-config-form">
    <h3>{{ i18n.t("frp.setup") }}</h3>

    <div class="form">
      <div class="form-group">
        <label>{{ i18n.t("frp.provider") }}</label>
        <SLSelect
          :model-value="provider"
          :options="providerOptions"
          @update:model-value="provider = $event"
        />
      </div>

      <div class="form-row">
        <div class="form-group">
          <label>{{ i18n.t("frp.server_addr") }}</label>
          <SLInput v-model="serverAddr" placeholder="frp.example.com" />
        </div>
        <div class="form-group" style="width: 120px">
          <label>{{ i18n.t("frp.server_port") }}</label>
          <SLInput v-model.number="serverPort" type="number" />
        </div>
      </div>

      <div class="form-group">
        <label>{{ i18n.t("frp.token") }}</label>
        <SLInput v-model="token" type="password" :placeholder="i18n.t('frp.token')" />
      </div>

      <div class="form-group">
        <label>{{ i18n.t("frp.tunnel_type") }}</label>
        <SLSelect
          :model-value="tunnelType"
          :options="tunnelTypeOptions"
          @update:model-value="tunnelType = $event"
        />
      </div>

      <div class="form-row">
        <div class="form-group">
          <label>{{ i18n.t("frp.local_port") }}</label>
          <SLInput v-model.number="localPort" type="number" />
        </div>
        <div class="form-group">
          <label>{{ i18n.t("frp.remote_port") }}</label>
          <SLInput v-model.number="remotePort" type="number" />
        </div>
      </div>

      <div class="form-group row">
        <label>{{ i18n.t("frp.auto_start") }}</label>
        <SLSwitch v-model="autoStart" />
      </div>

      <SLButton type="primary" :loading="saving" @click="handleSave">
        {{ i18n.t("frp.save_config") }}
      </SLButton>
    </div>
  </div>
</template>

<style scoped>
.frp-config-form {
  background: var(--bg-secondary);
  border-radius: 8px;
  border: 1px solid var(--border-color);
  padding: 20px;
}

.frp-config-form h3 {
  margin: 0 0 16px 0;
  font-size: 16px;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}

.form-group.row {
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
}

.form-group label {
  font-size: 13px;
  font-weight: 500;
}

.form-row {
  display: flex;
  gap: 12px;
}
</style>
