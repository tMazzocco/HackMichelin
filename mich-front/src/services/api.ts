import axios from "axios";

const api = axios.create({ baseURL: "", timeout: 10000 });

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
