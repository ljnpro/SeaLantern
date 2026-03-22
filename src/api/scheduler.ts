import { tauriInvoke } from "@api/tauri";

export interface ScheduledTask {
  id: string;
  server_id: string;
  name: string;
  enabled: boolean;
  task_type: TaskType;
  schedule: TaskSchedule;
  created_at: number;
  last_run_at: number | null;
  last_run_result: string | null;
}

export type TaskType =
  | "RestartServer"
  | { RunBackup: { incremental: boolean; include_paths: string[] } }
  | { ExecuteCommand: { command: string } };

export type TaskSchedule =
  | { Interval: { seconds: number } }
  | { Daily: { hour: number; minute: number } };

export interface TaskExecutionLog {
  id: string;
  task_id: string;
  executed_at: number;
  result: string;
  duration_ms: number;
}

export const schedulerApi = {
  async createTask(
    serverId: string,
    name: string,
    taskType: TaskType,
    schedule: TaskSchedule,
  ): Promise<ScheduledTask> {
    return tauriInvoke("create_scheduled_task", {
      serverId,
      name,
      taskType,
      schedule,
    });
  },

  async updateTask(task: ScheduledTask): Promise<ScheduledTask> {
    return tauriInvoke("update_scheduled_task", { task });
  },

  async deleteTask(taskId: string): Promise<void> {
    return tauriInvoke("delete_scheduled_task", { taskId });
  },

  async listTasks(serverId: string): Promise<ScheduledTask[]> {
    return tauriInvoke("list_scheduled_tasks", { serverId });
  },

  async toggleTask(taskId: string, enabled: boolean): Promise<void> {
    return tauriInvoke("toggle_scheduled_task", { taskId, enabled });
  },

  async getExecutionLogs(taskId: string): Promise<TaskExecutionLog[]> {
    return tauriInvoke("get_task_execution_logs", { taskId });
  },

  async runTaskNow(taskId: string): Promise<void> {
    return tauriInvoke("run_task_now", { taskId });
  },
};
