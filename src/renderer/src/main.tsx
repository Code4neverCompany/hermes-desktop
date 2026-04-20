import "./assets/main.css";

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import { I18nProvider } from "./components/I18nProvider";
import {
  createElectronDesktopClient,
  createElectronDesktopRuntime,
} from "../../shared/desktop/electron";
import { installDesktopGlobals } from "../../shared/desktop/globals";

installDesktopGlobals(
  createElectronDesktopClient(),
  createElectronDesktopRuntime(),
);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <I18nProvider>
      <App />
    </I18nProvider>
  </StrictMode>,
);
