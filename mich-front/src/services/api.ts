import axios from "axios";

// VITE_BACK_URL points to the Nginx API gateway (e.g. http://localhost:80).
// When set, axios uses it as the base for every request and the Vite dev proxy
// is bypassed. When absent (empty string), relative paths are used and the Vite
// proxy routes each /api/* prefix to the correct microservice port.
const baseURL = import.meta.env.VITE_BACK_URL ?? "";

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
