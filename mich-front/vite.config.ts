import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import { fileURLToPath } from "url";

// ES-module equivalent of __dirname
const __dirname = fileURLToPath(new URL(".", import.meta.url));

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(({ mode }) => {
  // loadEnv reads .env at config time so VITE_BACK_URL is available before the bundle
  const env = loadEnv(mode, __dirname, "");
  const backUrl = env.VITE_BACK_URL || "http://localhost:80";

  return {
    plugins: [react()],
    resolve: {
      alias: { "@": path.resolve(__dirname, "./src") },
    },
    clearScreen: false,
    server: {
      port: 1420,
      strictPort: true,
      host: host || false,
      hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
      watch: { ignored: ["**/src-tauri/**"] },
      proxy: {
        // All /api/* requests are forwarded to the gateway (server-to-server, no CORS)
        "/api": {
          target: backUrl,
          changeOrigin: true,
          // No path rewrite — the gateway expects the full /api/<service>/... path
        },
      },
    },
  };
});
