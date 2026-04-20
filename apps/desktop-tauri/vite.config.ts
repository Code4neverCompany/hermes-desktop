import { resolve } from "path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

const repoRoot = resolve(__dirname, "../..");

export default defineConfig({
  plugins: [tailwindcss(), react()],
  resolve: {
    dedupe: ["react", "react-dom"],
    alias: {
      "@renderer": resolve(repoRoot, "src/renderer/src"),
      react: resolve(repoRoot, "node_modules/react"),
      "react-dom": resolve(repoRoot, "node_modules/react-dom"),
      "react/jsx-runtime": resolve(repoRoot, "node_modules/react/jsx-runtime.js"),
      "react/jsx-dev-runtime": resolve(
        repoRoot,
        "node_modules/react/jsx-dev-runtime.js",
      ),
    },
  },
  server: {
    port: 1420,
    strictPort: true,
    fs: {
      allow: [
        repoRoot,
        resolve(repoRoot, "src"),
        resolve(repoRoot, "src/renderer"),
        resolve(repoRoot, "src/renderer/src"),
        resolve(repoRoot, "src/shared"),
        resolve(repoRoot, "build"),
        resolve(repoRoot, "resources"),
      ],
    },
  },
  build: {
    outDir: "dist",
  },
});
