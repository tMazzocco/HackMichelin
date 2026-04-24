# mich-front

React 19 frontend for **HackMichelin** — a Michelin-style restaurant review app.

## Stack

| Tool | Version | Role |
|---|---|---|
| React | 19 | UI framework |
| TypeScript | 5.8 | Type safety |
| Vite | 7 | Dev server + bundler |
| Mantine UI | 9 | Component library |
| Tailwind CSS | 3 | Utility classes |
| React Router | 6 | Client-side routing |
| React Leaflet | 4 | Interactive map |
| Axios | 1.7 | HTTP client |
| Lucide React | — | Icons |

## Pages & Routes

| Route | Page | Description |
|---|---|---|
| `/` | HomePage | Feed of nearby restaurants & posts |
| `/map` | MapPage | Interactive Leaflet map with restaurant pins |
| `/shorts` | ShortsPage | Vertical scroll short-form video/posts |
| `/articles` | ArticlesPage | Editorial articles list |
| `/articles/:id` | ArticleDetailPage | Article detail |
| `/restaurant/:id` | RestaurantDetailPage | Restaurant detail + posts |
| `/profile` | ProfilePage | User profile, follows & starred restaurants |

## Project Structure

```
src/
├── components/
│   ├── common/       # Reusable UI (RestaurantCard, ArticleCard, MichelinStar…)
│   ├── home/         # Home-specific components (ExperiencesTriptych…)
│   ├── layout/       # TopBar, BottomNav, ResizableSplit
│   └── map/          # MapView, MapErrorBoundary
├── context/          # AppContext (global auth/user state)
├── data/             # Static data (articles…)
├── hooks/            # Custom hooks (useGeolocation…)
├── pages/            # Page-level components
├── services/         # API clients (api.ts, restaurants.ts, posts.ts)
├── types/            # Shared TypeScript types
└── App.tsx           # Router + layout shell
```

## API Proxy

All requests to `/api/*` are proxied to the backend gateway:

- **Dev:** Vite proxy forwards to `VITE_BACK_URL` (see `.env`)
- **Prod:** Nginx reverse proxy (see `nginx.conf`)

No CORS issues — requests are always same-origin from the browser's perspective.

## Environment Variables

Create a `.env` file at the root of `mich-front/`:

```env
VITE_BACK_URL=http://localhost:80   # Backend gateway URL
```

## Getting Started

```bash
# Install dependencies
bun install

# Start dev server (http://localhost:1420)
bun dev

# Type-check + build for production
bun run build

# Preview production build locally
bun preview
```

## Docker

A `Dockerfile` and `nginx.conf` are included for production builds. The image serves the static bundle and proxies `/api/*` to the backend.

```bash
docker build -t mich-front .
docker run -p 8080:80 mich-front
```
