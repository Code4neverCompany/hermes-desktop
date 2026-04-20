export interface InstallStatus {
  installed: boolean;
  configured: boolean;
  hasApiKey: boolean;
  verified: boolean;
}

export interface InstallProgress {
  step: number;
  totalSteps: number;
  title: string;
  detail: string;
  log: string;
}

export interface ChatUsage {
  promptTokens: number;
  completionTokens: number;
  totalTokens: number;
  cost?: number;
  rateLimitRemaining?: number;
  rateLimitReset?: number;
}

export interface ChatHistoryMessage {
  role: string;
  content: string;
}

export interface SessionSummary {
  id: string;
  source: string;
  startedAt: number;
  endedAt: number | null;
  messageCount: number;
  model: string;
  title: string | null;
  preview: string;
}

export interface SessionMessage {
  id: number;
  role: "user" | "assistant";
  content: string;
  timestamp: number;
}

export interface ProfileSummary {
  name: string;
  path: string;
  isDefault: boolean;
  isActive: boolean;
  model: string;
  provider: string;
  hasEnv: boolean;
  hasSoul: boolean;
  skillCount: number;
  gatewayRunning: boolean;
}

export interface MemoryState {
  memory: {
    content: string;
    exists: boolean;
    lastModified: number | null;
    entries: Array<{ index: number; content: string }>;
    charCount: number;
    charLimit: number;
  };
  user: {
    content: string;
    exists: boolean;
    lastModified: number | null;
    charCount: number;
    charLimit: number;
  };
  stats: { totalSessions: number; totalMessages: number };
}

export interface ToolsetInfo {
  key: string;
  label: string;
  description: string;
  enabled: boolean;
}

export interface InstalledSkill {
  name: string;
  category: string;
  description: string;
  path: string;
}

export interface BundledSkill {
  name: string;
  description: string;
  category: string;
  source: string;
  installed: boolean;
}

export interface CachedSession {
  id: string;
  title: string;
  startedAt: number;
  source: string;
  messageCount: number;
  model: string;
}

export interface SessionSearchResult {
  sessionId: string;
  title: string | null;
  startedAt: number;
  source: string;
  messageCount: number;
  model: string;
  snippet: string;
}

export interface CredentialPoolEntry {
  key: string;
  label: string;
}

export interface ModelRecord {
  id: string;
  name: string;
  provider: string;
  model: string;
  baseUrl: string;
  createdAt: number;
}

export interface Claw3dStatus {
  cloned: boolean;
  installed: boolean;
  devServerRunning: boolean;
  adapterRunning: boolean;
  port: number;
  portInUse: boolean;
  wsUrl: string;
  running: boolean;
  error: string;
}

export interface UpdateAvailableInfo {
  version: string;
  releaseNotes: string;
}

export interface UpdateDownloadProgressInfo {
  percent: number;
}

export interface CronJob {
  id: string;
  name: string;
  schedule: string;
  prompt: string;
  state: "active" | "paused" | "completed";
  enabled: boolean;
  next_run_at: string | null;
  last_run_at: string | null;
  last_status: string | null;
  last_error: string | null;
  repeat: { times: number | null; completed: number } | null;
  deliver: string[];
  skills: string[];
  script: string | null;
}

export interface ActionResult {
  success: boolean;
  error?: string;
}

export interface BackupResult extends ActionResult {
  path?: string;
}

export interface MemoryProvider {
  name: string;
  description: string;
  installed: boolean;
  active: boolean;
  envVars: string[];
}

export interface McpServer {
  name: string;
  type: string;
  enabled: boolean;
  detail: string;
}

export interface LogReadResult {
  content: string;
  path: string;
}

export interface RuntimeVersions {
  electron?: string;
  chrome?: string;
  node?: string;
  tauri?: string;
  webview?: string;
}

export interface DesktopRuntime {
  shell: "electron" | "tauri";
  platform: string;
  versions: RuntimeVersions;
  officeEmbedding: "embedded" | "window";
}

export interface DesktopClient {
  checkInstall: () => Promise<InstallStatus>;
  startInstall: () => Promise<ActionResult>;
  onInstallProgress: (callback: (progress: InstallProgress) => void) => () => void;
  getHermesVersion: () => Promise<string | null>;
  refreshHermesVersion: () => Promise<string | null>;
  runHermesDoctor: () => Promise<string>;
  runHermesUpdate: () => Promise<ActionResult>;
  checkOpenClaw: () => Promise<{ found: boolean; path: string | null }>;
  runClawMigrate: () => Promise<ActionResult>;
  getLocale: () => Promise<"en">;
  setLocale: (locale: "en") => Promise<"en">;
  getEnv: (profile?: string) => Promise<Record<string, string>>;
  setEnv: (key: string, value: string, profile?: string) => Promise<boolean>;
  getConfig: (key: string, profile?: string) => Promise<string | null>;
  setConfig: (key: string, value: string, profile?: string) => Promise<boolean>;
  getHermesHome: (profile?: string) => Promise<string>;
  getModelConfig: (
    profile?: string,
  ) => Promise<{ provider: string; model: string; baseUrl: string }>;
  setModelConfig: (
    provider: string,
    model: string,
    baseUrl: string,
    profile?: string,
  ) => Promise<boolean>;
  sendMessage: (
    message: string,
    profile?: string,
    resumeSessionId?: string,
    history?: ChatHistoryMessage[],
  ) => Promise<{ response: string; sessionId?: string }>;
  abortChat: () => Promise<void>;
  onChatChunk: (callback: (chunk: string) => void) => () => void;
  onChatDone: (callback: (sessionId?: string) => void) => () => void;
  onChatToolProgress: (callback: (tool: string) => void) => () => void;
  onChatUsage: (callback: (usage: ChatUsage) => void) => () => void;
  onChatError: (callback: (error: string) => void) => () => void;
  startGateway: () => Promise<boolean>;
  stopGateway: () => Promise<boolean>;
  gatewayStatus: () => Promise<boolean>;
  getPlatformEnabled: (profile?: string) => Promise<Record<string, boolean>>;
  setPlatformEnabled: (
    platform: string,
    enabled: boolean,
    profile?: string,
  ) => Promise<boolean>;
  listSessions: (limit?: number, offset?: number) => Promise<SessionSummary[]>;
  getSessionMessages: (sessionId: string) => Promise<SessionMessage[]>;
  listProfiles: () => Promise<ProfileSummary[]>;
  createProfile: (name: string, clone: boolean) => Promise<ActionResult>;
  deleteProfile: (name: string) => Promise<ActionResult>;
  setActiveProfile: (name: string) => Promise<boolean>;
  readMemory: (profile?: string) => Promise<MemoryState>;
  addMemoryEntry: (content: string, profile?: string) => Promise<ActionResult>;
  updateMemoryEntry: (
    index: number,
    content: string,
    profile?: string,
  ) => Promise<ActionResult>;
  removeMemoryEntry: (index: number, profile?: string) => Promise<boolean>;
  writeUserProfile: (content: string, profile?: string) => Promise<ActionResult>;
  readSoul: (profile?: string) => Promise<string>;
  writeSoul: (content: string, profile?: string) => Promise<boolean>;
  resetSoul: (profile?: string) => Promise<string>;
  getToolsets: (profile?: string) => Promise<ToolsetInfo[]>;
  setToolsetEnabled: (
    key: string,
    enabled: boolean,
    profile?: string,
  ) => Promise<boolean>;
  listInstalledSkills: (profile?: string) => Promise<InstalledSkill[]>;
  listBundledSkills: () => Promise<BundledSkill[]>;
  getSkillContent: (skillPath: string) => Promise<string>;
  installSkill: (identifier: string, profile?: string) => Promise<ActionResult>;
  uninstallSkill: (name: string, profile?: string) => Promise<ActionResult>;
  listCachedSessions: (limit?: number, offset?: number) => Promise<CachedSession[]>;
  syncSessionCache: () => Promise<CachedSession[]>;
  updateSessionTitle: (sessionId: string, title: string) => Promise<void>;
  searchSessions: (query: string, limit?: number) => Promise<SessionSearchResult[]>;
  getCredentialPool: () => Promise<Record<string, CredentialPoolEntry[]>>;
  setCredentialPool: (
    provider: string,
    entries: CredentialPoolEntry[],
  ) => Promise<boolean>;
  listModels: () => Promise<ModelRecord[]>;
  addModel: (
    name: string,
    provider: string,
    model: string,
    baseUrl: string,
  ) => Promise<ModelRecord>;
  removeModel: (id: string) => Promise<boolean>;
  updateModel: (id: string, fields: Record<string, string>) => Promise<boolean>;
  claw3dStatus: () => Promise<Claw3dStatus>;
  claw3dSetup: () => Promise<ActionResult>;
  onClaw3dSetupProgress: (callback: (progress: InstallProgress) => void) => () => void;
  claw3dGetPort: () => Promise<number>;
  claw3dSetPort: (port: number) => Promise<boolean>;
  claw3dGetWsUrl: () => Promise<string>;
  claw3dSetWsUrl: (url: string) => Promise<boolean>;
  claw3dStartAll: () => Promise<ActionResult>;
  claw3dStopAll: () => Promise<boolean>;
  claw3dGetLogs: () => Promise<string>;
  claw3dStartDev: () => Promise<boolean>;
  claw3dStopDev: () => Promise<boolean>;
  claw3dStartAdapter: () => Promise<boolean>;
  claw3dStopAdapter: () => Promise<boolean>;
  checkForUpdates: () => Promise<string | null>;
  downloadUpdate: () => Promise<boolean>;
  installUpdate: () => Promise<void>;
  getAppVersion: () => Promise<string>;
  onUpdateAvailable: (callback: (info: UpdateAvailableInfo) => void) => () => void;
  onUpdateDownloadProgress: (
    callback: (info: UpdateDownloadProgressInfo) => void,
  ) => () => void;
  onUpdateDownloaded: (callback: () => void) => () => void;
  onMenuNewChat: (callback: () => void) => () => void;
  onMenuSearchSessions: (callback: () => void) => () => void;
  listCronJobs: (includeDisabled?: boolean, profile?: string) => Promise<CronJob[]>;
  createCronJob: (
    schedule: string,
    prompt?: string,
    name?: string,
    deliver?: string,
    profile?: string,
  ) => Promise<ActionResult>;
  removeCronJob: (jobId: string, profile?: string) => Promise<ActionResult>;
  pauseCronJob: (jobId: string, profile?: string) => Promise<ActionResult>;
  resumeCronJob: (jobId: string, profile?: string) => Promise<ActionResult>;
  triggerCronJob: (jobId: string, profile?: string) => Promise<ActionResult>;
  openExternal: (url: string) => Promise<void>;
  openOfficeWindow?: (url: string) => Promise<void>;
  runHermesBackup: (profile?: string) => Promise<BackupResult>;
  runHermesImport: (archivePath: string, profile?: string) => Promise<ActionResult>;
  runHermesDump: () => Promise<string>;
  discoverMemoryProviders: (profile?: string) => Promise<MemoryProvider[]>;
  listMcpServers: (profile?: string) => Promise<McpServer[]>;
  readLogs: (logFile?: string, lines?: number) => Promise<LogReadResult>;
}
