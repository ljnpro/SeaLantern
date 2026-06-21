import { tauriInvoke } from "@api/tauri";

export interface ResourceSearchResult {
  id: string;
  name: string;
  slug: string;
  summary: string;
  author: string;
  downloads: number;
  icon_url: string | null;
  source: "Modrinth" | "CurseForge";
  project_type: "Mod" | "Plugin" | "DataPack" | "Shader" | "ResourcePack";
  categories: string[];
  latest_game_versions: string[];
  date_modified: string;
}

export interface ResourceVersion {
  id: string;
  version_number: string;
  name: string;
  game_versions: string[];
  loaders: string[];
  download_url: string;
  file_name: string;
  file_size: number;
  dependencies: ResourceDependency[];
  date_published: string;
}

export interface ResourceDependency {
  project_id: string;
  dependency_type: string;
}

export interface ResourceDetail {
  id: string;
  name: string;
  description: string;
  source: "Modrinth" | "CurseForge";
  downloads: number;
  icon_url: string | null;
  source_url: string | null;
  wiki_url: string | null;
  versions: ResourceVersion[];
}

export interface InstalledResource {
  id: string;
  project_id: string;
  source: "Modrinth" | "CurseForge";
  name: string;
  version_id: string;
  version_number: string;
  file_name: string;
  installed_at: number;
  server_id: string;
}

export interface ResourceUpdateInfo {
  resource_id: string;
  project_id: string;
  current_version: string;
  latest_version_id: string;
  latest_version_number: string;
  source: "Modrinth" | "CurseForge";
  name: string;
}

export const resourceApi = {
  async search(
    query: string,
    gameVersion: string | null,
    loader: string | null,
    projectType: string | null,
    source: string | null,
    offset: number,
    limit: number,
  ): Promise<ResourceSearchResult[]> {
    return tauriInvoke("search_resources", {
      query,
      gameVersion,
      loader,
      projectType,
      source,
      offset,
      limit,
    });
  },

  async getDetail(source: string, projectId: string): Promise<ResourceDetail> {
    return tauriInvoke("get_resource_detail", { source, projectId });
  },

  async getVersions(
    source: string,
    projectId: string,
    gameVersion: string | null,
    loader: string | null,
  ): Promise<ResourceVersion[]> {
    return tauriInvoke("get_resource_versions", {
      source,
      projectId,
      gameVersion,
      loader,
    });
  },

  async install(
    serverId: string,
    source: string,
    projectId: string,
    versionId: string,
  ): Promise<InstalledResource> {
    return tauriInvoke("install_resource", {
      serverId,
      source,
      projectId,
      versionId,
    });
  },

  async getInstalled(serverId: string): Promise<InstalledResource[]> {
    return tauriInvoke("get_installed_resources", { serverId });
  },

  async uninstall(serverId: string, resourceId: string): Promise<void> {
    return tauriInvoke("uninstall_resource", { serverId, resourceId });
  },

  async checkUpdates(serverId: string): Promise<ResourceUpdateInfo[]> {
    return tauriInvoke("check_resource_updates", { serverId });
  },

  async update(
    serverId: string,
    resourceId: string,
    versionId: string,
  ): Promise<InstalledResource> {
    return tauriInvoke("update_resource", { serverId, resourceId, versionId });
  },
};
