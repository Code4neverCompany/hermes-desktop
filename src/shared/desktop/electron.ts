import type { DesktopClient, DesktopRuntime } from "./types";

interface ElectronProcessInfo {
  platform: string;
  versions: {
    electron?: string;
    chrome?: string;
    node?: string;
  };
}

interface ElectronWindowLike {
  electron?: {
    process?: ElectronProcessInfo;
  };
  hermesAPI: DesktopClient;
}

export function createElectronDesktopClient(
  win: ElectronWindowLike = window as unknown as ElectronWindowLike,
): DesktopClient {
  return {
    ...win.hermesAPI,
    openOfficeWindow: async (url: string) => {
      await win.hermesAPI.openExternal(url);
    },
  };
}

export function createElectronDesktopRuntime(
  win: ElectronWindowLike = window as unknown as ElectronWindowLike,
): DesktopRuntime {
  const processInfo = win.electron?.process;
  return {
    shell: "electron",
    platform: processInfo?.platform || "unknown",
    versions: {
      electron: processInfo?.versions?.electron,
      chrome: processInfo?.versions?.chrome,
      node: processInfo?.versions?.node,
    },
    officeEmbedding: "embedded",
  };
}
