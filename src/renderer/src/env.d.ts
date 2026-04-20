/// <reference types="vite/client" />

import type {
  DesktopClient,
  DesktopRuntime,
} from "../../shared/desktop/types";

declare global {
  interface Window {
    desktopClient: DesktopClient;
    desktopRuntime: DesktopRuntime;
  }

  const desktopClient: DesktopClient;
  const desktopRuntime: DesktopRuntime;
}

export {};
