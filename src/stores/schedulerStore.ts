import { defineStore } from "pinia";
import { ref } from "vue";
import { schedulerApi } from "@api/scheduler";
import type {
  ScheduledTask,
  TaskType,
  TaskSchedule,
  TaskExecutionLog,
} from "@api/scheduler";
import { useLoading } from "@composables/useAsync";

export const useSchedulerStore = defineStore("scheduler", () => {
  const tasks = ref<ScheduledTask[]>([]);
  const executionLogs = ref<TaskExecutionLog[]>([]);
  const { loading, withLoading } = useLoading(false);
  const error = ref<string | null>(null);

  async function refreshTasks(serverId: string) {
    error.value = null;
    try {
      tasks.value = await withLoading(() =>
        schedulerApi.listTasks(serverId),
      );
    } catch (e) {
      error.value = String(e);
    }
  }

  async function createTask(
    serverId: string,
    name: string,
    taskType: TaskType,
    schedule: TaskSchedule,
  ) {
    error.value = null;
    try {
      const task = await withLoading(() =>
        schedulerApi.createTask(serverId, name, taskType, schedule),
      );
      tasks.value.push(task);
      return task;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function updateTask(task: ScheduledTask) {
    error.value = null;
    try {
      const updated = await schedulerApi.updateTask(task);
      const idx = tasks.value.findIndex((t) => t.id === updated.id);
      if (idx >= 0) tasks.value[idx] = updated;
      return updated;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function deleteTask(taskId: string) {
    error.value = null;
    try {
      await schedulerApi.deleteTask(taskId);
      tasks.value = tasks.value.filter((t) => t.id !== taskId);
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function toggleTask(taskId: string, enabled: boolean) {
    error.value = null;
    try {
      await schedulerApi.toggleTask(taskId, enabled);
      const task = tasks.value.find((t) => t.id === taskId);
      if (task) task.enabled = enabled;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function runTaskNow(taskId: string) {
    error.value = null;
    try {
      await schedulerApi.runTaskNow(taskId);
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function loadExecutionLogs(taskId: string) {
    try {
      executionLogs.value = await schedulerApi.getExecutionLogs(taskId);
    } catch (e) {
      error.value = String(e);
    }
  }

  return {
    tasks,
    executionLogs,
    loading,
    error,
    refreshTasks,
    createTask,
    updateTask,
    deleteTask,
    toggleTask,
    runTaskNow,
    loadExecutionLogs,
  };
});
