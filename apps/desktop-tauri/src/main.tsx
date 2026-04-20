import "../../../src/renderer/src/assets/main.css";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "../../../src/renderer/src/App";
import { I18nProvider } from "../../../src/renderer/src/components/I18nProvider";
import { installDesktopGlobals } from "../../../src/shared/desktop/globals";
import {
  createTauriDesktopClient,
  tauriDesktopRuntime,
} from "./tauriClient";

installDesktopGlobals(createTauriDesktopClient(), tauriDesktopRuntime);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <I18nProvider>
      <App />
    </I18nProvider>
  </StrictMode>,
);
