# HackMichelin

Michelin restaurant discovery platform built as a microservice architecture.
Data covers the full Michelin guide dataset (PostgreSQL + Elasticsearch).
Media (photos, videos) is served as HLS streams.

---

## Architecture Overview

```
Browser / Mobile App
        │
        ▼
    Nginx :80/:443          ← reverse proxy / gateway
        │
        ├── /api/maps/      → MapsDataService  :3000  (Rust / Axum)
        ├── /api/download/  → DownloadService  :3001  (Rust / Axum)
        ├── /api/login/     → LoginService     :????  (TBD)
        ├── /api/users/     → UserService      :????  (TBD)
        ├── /api/search/    → SearchService    :????  (TBD)
        ├── /api/upload/    → UploadService    :????  (TBD)
        ├── /api/comments/  → CommentService   :????  (TBD)
        ├── /api/likes/     → LikeService      :????  (TBD)
        └── /api/stats/     → StatsService     :????  (TBD)

Infrastructure:
  PostgreSQL    :5432   — restaurant master data
  Elasticsearch :9200   — full-text & faceted search
  Cassandra     :9042   — high-write data (comments, likes, stats)
  Mosquitto     :1883   — MQTT broker (service-to-service events)
  Kibana        :5601   — Elasticsearch UI
```

---

## Services

### MapsDataService (`MapsDataService/`)
Language: Rust (Tokio + Axum)  |  Port: 3000

Geographic restaurant queries against PostgreSQL.
Exposes an HTTP API and an MQTT request/reply interface.

Key endpoints:
- `GET /health`
- `GET /restaurants/nearby?lat=&lng=&radius=&limit=`
- `GET /restaurants/:id`

Via nginx: `GET /api/maps/restaurants/nearby?...`

See [MapsDataService/README](MapsDataService/README) for full API docs and MQTT interface.

---

### DownloadService (`DownloadService/`)
Language: Rust (Tokio + Axum)  |  Port: 3001

Serves media files (photos & videos) from the `media/` folder as HLS v3 VOD playlists (.m3u8) with full Range-request support for video seeking.

Key endpoints:
- `GET /health`
- `GET /playlist.m3u8[?files=name1.mp4,name2.jpg]`  — HLS manifest
- `GET /files/:name`                                  — raw media file

Via nginx: `GET /api/download/playlist.m3u8`

Frontend players: use **hls.js** (all browsers) or native HLS (Safari only).

See [DownloadService/README](DownloadService/README) for full API docs and frontend integration.

---

### LoginService (`LoginService/`)
Status: **TBD**

Handles user authentication (registration, login, token issuance).

---

### UserService (`UserService/`)
Status: **TBD**

User profile management (read/update profile, preferences).

---

### SearchService (`SearchService/`)
Status: **TBD**

Full-text and faceted restaurant search backed by Elasticsearch.

---

### UploadService (`UploadService/`)
Status: **TBD**

Media file ingestion — accepts uploads and stores them in the `media/` folder for DownloadService to serve.

---

### CommentService (`CommentService/`)
Status: **TBD**

Restaurant comments stored in Cassandra (high write throughput).

---

### LikeService (`LikeService/`)
Status: **TBD**

Restaurant like/bookmark functionality stored in Cassandra.

---

### StatsService (`StatsService/`)
Status: **TBD**

Aggregated statistics (view counts, popularity scores) stored in Cassandra.

---

### Frontend (`mich-front/`)
Stack: React + TypeScript + Vite (Tauri wrapper available for desktop)

Development:
```powershell
cd mich-front
npm install
npm run dev
```

---

## Infrastructure

| Service       | Image                          | Port(s)       | Purpose                          |
|---------------|--------------------------------|---------------|----------------------------------|
| nginx         | nginx:alpine                   | 80, 443       | Reverse proxy / API gateway      |
| postgres      | postgres:16-alpine             | 5432          | Restaurant master data           |
| elasticsearch | elasticsearch:8.13.0           | 9200, 9300    | Full-text search                 |
| kibana        | kibana:8.13.0                  | 5601          | Elasticsearch UI                 |
| cassandra     | cassandra:5                    | 9042          | Comments, likes, stats           |
| mosquitto     | eclipse-mosquitto:2            | 1883, 9001    | MQTT broker                      |
| importer      | custom (init/import)           | —             | Seeds PG + ES from JSONL on boot |

---

## Quick Start

### Prerequisites
- Docker Desktop
- Rust (for local development of Rust services)
- Node.js 18+ (for frontend)

### Start all infrastructure + implemented services

```powershell
# From project root
docker compose up -d
```

> Note: `download_service` is commented out in docker-compose.yml by default.
> Uncomment the `download_service` block to enable it.

### Start DownloadService locally

```powershell
cd DownloadService
cargo run
# Service available at http://localhost:3001
```

### Start MapsDataService locally

```powershell
# Requires PostgreSQL and Mosquitto running
docker compose up postgres mosquitto -d

cd MapsDataService
cargo run
# Service available at http://localhost:3000
```

### Seed data

The `importer` container runs automatically on `docker compose up` and loads `ressources/all_restaurants.jsonl` into PostgreSQL and Elasticsearch.

---

## Environment Variables

Copy `.env.example` to `.env` in each service folder and adjust as needed.

| Variable          | Default         | Description                     |
|-------------------|-----------------|---------------------------------|
| POSTGRES_USER     | admin           | PostgreSQL username             |
| POSTGRES_PASSWORD | changeme        | PostgreSQL password             |
| POSTGRES_DB       | hackmichelin    | PostgreSQL database name        |

See each service README for service-specific variables.

---

## Media Files

Drop media files into the `media/` folder at the project root.
They are immediately available via DownloadService without rebuilding.

Supported formats: `jpg jpeg png gif webp bmp mp4 ts m4s mov avi mkv webm`
