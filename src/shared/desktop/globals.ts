import type { DesktopClient, DesktopRuntime } from "./types";

export function installDesktopGlobals(
  client: DesktopClient,
  runtime: DesktopRuntime,
): void {
  const win = window as unknown as Window & {
    desktopClient: DesktopClient;
    desktopRuntime: DesktopRuntime;
  };
  const target = globalThis as typeof globalThis & {
    desktopClient: DesktopClient;
    desktopRuntime: DesktopRuntime;
  };

  win.desktopClient = client;
  win.desktopRuntime = runtime;
  target.desktopClient = client;
  target.desktopRuntime = runtime;
}
