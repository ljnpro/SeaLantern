<script setup lang="ts">
import { ref } from "vue";
import { useBackupStore } from "@stores/backupStore";
import { i18n } from "@language";
import SLModal from "@components/common/SLModal.vue";
import SLInput from "@components/common/SLInput.vue";
import SLSwitch from "@components/common/SLSwitch.vue";
import SLButton from "@components/common/SLButton.vue";

const props = defineProps<{
  serverId: string;
}>();

const emit = defineEmits<{
  close: [];
  created: [];
}>();

const backupStore = useBackupStore();
const name = ref("");
const incremental = ref(false);
const includePaths = ref("world");
const creating = ref(false);

async function handleCreate() {
  creating.value = true;
  try {
    const paths = includePaths.value
      .split(",")
      .map((p) => p.trim())
      .filter(Boolean);
    await backupStore.createBackup(props.serverId, name.value || null, incremental.value, paths);
    emit("created");
  } catch {
    // error handled by store
  } finally {
    creating.value = false;
  }
}
</script>

<template>
  <SLModal :title="i18n.t('backup.create')" @close="emit('close')">
    <div class="form">
      <div class="form-group">
        <label>{{ i18n.t("backup.name") }}</label>
        <SLInput v-model="name" :placeholder="i18n.t('backup.name')" />
      </div>

      <div class="form-group">
        <label>{{ i18n.t("backup.include_paths") }}</label>
        <SLInput v-model="includePaths" placeholder="world, world_nether, world_the_end" />
      </div>

      <div class="form-group row">
        <label>{{ i18n.t("backup.incremental") }}</label>
        <SLSwitch v-model="incremental" />
      </div>
    </div>

    <template #footer>
      <SLButton @click="emit('close')">{{ i18n.t("common.cancel") }}</SLButton>
      <SLButton type="primary" :loading="creating" @click="handleCreate">
        {{ i18n.t("backup.create") }}
      </SLButton>
    </template>
  </SLModal>
</template>

<style scoped>
.form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group.row {
  flex-direction: row;
  align-items: center;
  justify-content: space-between;
}

.form-group label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}
</style>
