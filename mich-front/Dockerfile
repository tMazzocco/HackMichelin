# =============================================================================
# Stage 1 — Build the Vite/React web app
# =============================================================================
FROM oven/bun:1 AS builder

WORKDIR /app

# Copy manifests first for layer-cache efficiency
COPY package.json bun.lockb* ./

# --ignore-scripts skips the @tauri-apps/cli native-binary postinstall, which
# is not needed for a plain web build (tsc + vite).
# All other packages (vite, typescript, react, @tauri-apps/api …) work fine.
RUN bun install --frozen-lockfile --ignore-scripts

# Copy sources — src-tauri/ is excluded via .dockerignore
COPY . .

# Build the web bundle
RUN bun run build

# =============================================================================
# Stage 2 — Serve with nginx
# =============================================================================
FROM nginx:1.27-alpine AS runner

# Replace default config with our SPA-aware one
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy the compiled assets from the builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

# Render (and most PaaS) expects port 80
EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
