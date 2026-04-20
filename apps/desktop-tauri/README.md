# Hermes Desktop Tauri

This folder contains the side-by-side Tauri migration for Hermes Desktop.

Current state:
- Reuses the shared React renderer from the Electron app
- Installs a shared `desktopClient` / `desktopRuntime` boundary
- Boots in a Tauri shell with an initial Rust command surface
- Keeps the existing Electron app untouched

Next migration steps:
- Port the remaining desktop commands from `src/main/*`
- Add full chat/gateway/process orchestration
- Replace placeholder/stub methods in `src/tauriClient.ts`
- Reach feature parity before removing Electron code
