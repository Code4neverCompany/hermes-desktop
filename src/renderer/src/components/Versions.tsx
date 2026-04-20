import { useState } from "react";

function Versions(): React.JSX.Element {
  const [versions] = useState(desktopRuntime.versions);
  const shellLabel =
    desktopRuntime.shell === "tauri" ? "Tauri" : "Electron";

  return (
    <ul className="versions">
      <li className="electron-version">
        {shellLabel} v{versions.tauri || versions.electron || "unknown"}
      </li>
      <li className="chrome-version">
        Webview v{versions.webview || versions.chrome || "unknown"}
      </li>
      <li className="node-version">Node v{versions.node || "n/a"}</li>
    </ul>
  );
}

export default Versions;
