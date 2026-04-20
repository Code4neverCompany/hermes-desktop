import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  ActionResult,
  BackupResult,
  BundledSkill,
  CachedSession,
  ChatHistoryMessage,
  ChatUsage,
  Claw3dStatus,
  CredentialPoolEntry,
  CronJob,
  DesktopClient,
  DesktopRuntime,
  InstallProgress,
  InstallStatus,
  InstalledSkill,
  LogReadResult,
  McpServer,
  MemoryProvider,
  MemoryState,
  ModelRecord,
  ProfileSummary,
  SessionMessage,
  SessionSearchResult,
  SessionSummary,
  ToolsetInfo,
  UpdateAvailableInfo,
  UpdateDownloadProgressInfo,
} from "../../../src/shared/desktop/types";

function unimplemented(name: string): never {
  throw new Error(`Tauri migration preview: \`${name}\` is not implemented yet.`);
}

function subscribe<T>(event: string, callback: (payload: T) => void): () => void {
  let cleanup: (() => void) | null = null;
  void listen<T>(event, (evt) => callback(evt.payload)).then((unlisten) => {
    cleanup = unlisten;
  });
  return () => {
    cleanup?.();
  };
}

const fallbackClient = new Proxy(
  {},
  {
    get: (_target, prop) => {
      const name = String(prop);
      if (name.startsWith("on")) {
        return () => () => {};
      }
      return () => Promise.reject(new Error(`Tauri migration preview: \`${name}\` is not implemented yet.`));
    },
  },
) as DesktopClient;

export const tauriDesktopRuntime: DesktopRuntime = {
  shell: "tauri",
  platform: navigator.userAgent.includes("Mac")
    ? "darwin"
    : navigator.userAgent.includes("Windows")
      ? "win32"
      : navigator.userAgent.includes("Linux")
        ? "linux"
        : "unknown",
  versions: {
    tauri: "preview",
    webview: navigator.userAgent,
  },
  officeEmbedding: "window",
};

export function createTauriDesktopClient(): DesktopClient {
  return {
    ...fallbackClient,
    checkInstall: () => invoke<InstallStatus>("check_install"),
    startInstall: async () => ({
      success: false,
      error: "Tauri installer flow is not implemented yet.",
    }),
    onInstallProgress: (callback) =>
      subscribe<InstallProgress>("install-progress", callback),
    getHermesVersion: () => invoke<string | null>("get_hermes_version"),
    refreshHermesVersion: () => invoke<string | null>("get_hermes_version"),
    runHermesDoctor: async () => unimplemented("runHermesDoctor"),
    runHermesUpdate: async () => ({
      success: false,
      error: "Tauri update flow is not implemented yet.",
    }),
    checkOpenClaw: async () => ({ found: false, path: null }),
    runClawMigrate: async () => ({
      success: false,
      error: "OpenClaw migration is not implemented yet.",
    }),
    getLocale: async () => "en",
    setLocale: async () => "en",
    getEnv: (profile) =>
      invoke<Record<string, string>>("get_env", { profile }),
    setEnv: (key, value, profile) =>
      invoke<boolean>("set_env", { key, value, profile }),
    getConfig: (key, profile) =>
      invoke<string | null>("get_config", { key, profile }),
    setConfig: (key, value, profile) =>
      invoke<boolean>("set_config", { key, value, profile }),
    getHermesHome: (profile) => invoke<string>("get_hermes_home", { profile }),
    getModelConfig: (profile) =>
      invoke<{ provider: string; model: string; baseUrl: string }>(
        "get_model_config",
        { profile },
      ),
    setModelConfig: (provider, model, baseUrl, profile) =>
      invoke<boolean>("set_model_config", {
        provider,
        model,
        baseUrl,
        profile,
      }),
    sendMessage: (message, profile, resumeSessionId, history) =>
      invoke<{ response: string; sessionId?: string }>("send_message", {
        message,
        profile,
        resumeSessionId,
        history: history as ChatHistoryMessage[] | undefined,
      }),
    abortChat: () => invoke<void>("abort_chat"),
    onChatChunk: (callback) => subscribe<string>("chat-chunk", callback),
    onChatDone: (callback) => subscribe<string | undefined>("chat-done", callback),
    onChatToolProgress: (callback) =>
      subscribe<string>("chat-tool-progress", callback),
    onChatUsage: (callback) => subscribe<ChatUsage>("chat-usage", callback),
    onChatError: (callback) => subscribe<string>("chat-error", callback),
    startGateway: () => invoke<boolean>("start_gateway"),
    stopGateway: () => invoke<boolean>("stop_gateway"),
    gatewayStatus: () => invoke<boolean>("gateway_status"),
    getPlatformEnabled: async () => unimplemented("getPlatformEnabled"),
    setPlatformEnabled: async () => unimplemented("setPlatformEnabled"),
    listSessions: (limit, offset) =>
      invoke<SessionSummary[]>("list_sessions", { limit, offset }),
    getSessionMessages: (sessionId) =>
      invoke<SessionMessage[]>("get_session_messages", { sessionId }),
    listProfiles: () => invoke<ProfileSummary[]>("list_profiles"),
    createProfile: (name, clone) =>
      invoke<ActionResult>("create_profile", { name, clone }),
    deleteProfile: (name) =>
      invoke<ActionResult>("delete_profile", { name }),
    setActiveProfile: (name) =>
      invoke<boolean>("set_active_profile", { name }),
    readMemory: (profile) =>
      invoke<MemoryState>("read_memory", { profile }),
    addMemoryEntry: (content, profile) =>
      invoke<ActionResult>("add_memory_entry", { content, profile }),
    updateMemoryEntry: (index, content, profile) =>
      invoke<ActionResult>("update_memory_entry", { index, content, profile }),
    removeMemoryEntry: (index, profile) =>
      invoke<boolean>("remove_memory_entry", { index, profile }),
    writeUserProfile: (content, profile) =>
      invoke<ActionResult>("write_user_profile", { content, profile }),
    readSoul: (profile) => invoke<string>("read_soul", { profile }),
    writeSoul: (content, profile) =>
      invoke<boolean>("write_soul", { content, profile }),
    resetSoul: (profile) => invoke<string>("reset_soul", { profile }),
    getToolsets: async () => [] as ToolsetInfo[],
    setToolsetEnabled: async () => unimplemented("setToolsetEnabled"),
    listInstalledSkills: async () => [] as InstalledSkill[],
    listBundledSkills: async () => [] as BundledSkill[],
    getSkillContent: async () => unimplemented("getSkillContent"),
    installSkill: async () => unimplemented("installSkill"),
    uninstallSkill: async () => unimplemented("uninstallSkill"),
    listCachedSessions: (limit, offset) =>
      invoke<CachedSession[]>("list_cached_sessions", { limit, offset }),
    syncSessionCache: () =>
      invoke<CachedSession[]>("sync_session_cache"),
    updateSessionTitle: (sessionId, title) =>
      invoke<void>("update_session_title", { sessionId, title }),
    searchSessions: (query, limit) =>
      invoke<SessionSearchResult[]>("search_sessions", { query, limit }),
    getCredentialPool: async () => ({} as Record<string, CredentialPoolEntry[]>),
    setCredentialPool: async () => unimplemented("setCredentialPool"),
    listModels: () => invoke<ModelRecord[]>("list_models"),
    addModel: async () => unimplemented("addModel"),
    removeModel: async () => unimplemented("removeModel"),
    updateModel: async () => unimplemented("updateModel"),
    claw3dStatus: () => invoke<Claw3dStatus>("claw3d_status"),
    claw3dSetup: async () => ({
      success: false,
      error: "Claw3D setup is not implemented yet.",
    }),
    onClaw3dSetupProgress: (callback) =>
      subscribe<InstallProgress>("claw3d-setup-progress", callback),
    claw3dGetPort: async () => (await invoke<Claw3dStatus>("claw3d_status")).port,
    claw3dSetPort: async () => unimplemented("claw3dSetPort"),
    claw3dGetWsUrl: async () => (await invoke<Claw3dStatus>("claw3d_status")).wsUrl,
    claw3dSetWsUrl: async () => unimplemented("claw3dSetWsUrl"),
    claw3dStartAll: async () => ({
      success: false,
      error: "Claw3D process orchestration is not implemented yet.",
    }),
    claw3dStopAll: async () => true,
    claw3dGetLogs: async () => "",
    claw3dStartDev: async () => unimplemented("claw3dStartDev"),
    claw3dStopDev: async () => unimplemented("claw3dStopDev"),
    claw3dStartAdapter: async () => unimplemented("claw3dStartAdapter"),
    claw3dStopAdapter: async () => unimplemented("claw3dStopAdapter"),
    checkForUpdates: async () => null,
    downloadUpdate: async () => false,
    installUpdate: async () => {},
    getAppVersion: () => invoke<string>("get_app_version"),
    onUpdateAvailable: (callback) =>
      subscribe<UpdateAvailableInfo>("update-available", callback),
    onUpdateDownloadProgress: (callback) =>
      subscribe<UpdateDownloadProgressInfo>("update-download-progress", callback),
    onUpdateDownloaded: (callback) =>
      subscribe<void>("update-downloaded", callback),
    onMenuNewChat: (callback) => subscribe<void>("menu-new-chat", callback),
    onMenuSearchSessions: (callback) =>
      subscribe<void>("menu-search-sessions", callback),
    listCronJobs: async () => [] as CronJob[],
    createCronJob: async () => unimplemented("createCronJob"),
    removeCronJob: async () => unimplemented("removeCronJob"),
    pauseCronJob: async () => unimplemented("pauseCronJob"),
    resumeCronJob: async () => unimplemented("resumeCronJob"),
    triggerCronJob: async () => unimplemented("triggerCronJob"),
    openExternal: (url) => invoke<void>("open_external", { url }),
    openOfficeWindow: (url) => invoke<void>("open_office_window", { url }),
    runHermesBackup: async () => ({ success: false } as BackupResult),
    runHermesImport: async () => unimplemented("runHermesImport"),
    runHermesDump: async () => unimplemented("runHermesDump"),
    discoverMemoryProviders: async () => [] as MemoryProvider[],
    listMcpServers: async () => [] as McpServer[],
    readLogs: async () => ({ content: "", path: "" } as LogReadResult),
  };
}
