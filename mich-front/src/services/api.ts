import axios from "axios";

// Use relative paths so requests go through the Vite dev proxy in development
// (which forwards /api/* to VITE_BACK_URL) and through Nginx in production.
// This avoids browser CORS restrictions entirely.
const baseURL = "";

const api = axios.create({ baseURL, timeout: 10000 });

api.interceptors.response.use(
  (res) => {
    // Reject silently-successful responses that contain HTML instead of JSON.
    // This happens in production when the nginx SPA server falls back to
    // index.html for unknown /api/* paths (no backend proxy configured).
    const ct = String(res.headers["content-type"] ?? res.headers.get?.("content-type") ?? "");
    if (!ct.includes("application/json")) {
      console.error(`[API] ${res.config.url} — expected JSON but got "${ct}"`);
      return Promise.reject(
        new Error(`Non-JSON response from ${res.config.url} (${ct})`)
      );
    }
    return res;
  },
  (err) => {
    const url = err?.config?.url ?? "unknown";
    const status = err?.response?.status ?? err?.code ?? err?.message;
    console.error(`[API] ${url} — ${status}`);
    return Promise.reject(err);
  }
);

export default api;
