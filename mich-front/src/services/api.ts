import axios from "axios";

// Use relative paths so requests go through the Vite dev proxy in development
// (which forwards /api/* to VITE_BACK_URL) and through Nginx in production.
// This avoids browser CORS restrictions entirely.
const baseURL = "";

const api = axios.create({ baseURL, timeout: 10000 });

api.interceptors.response.use(
  (res) => res,
  (err) => {
    const url = err?.config?.url ?? "unknown";
    const status = err?.response?.status ?? err?.code ?? err?.message;
    console.error(`[API] ${url} — ${status}`);
    return Promise.reject(err);
  }
);

export default api;
