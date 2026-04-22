# HackMichelin

Google Maps meets Instagram for the Michelin Guide.
Explore restaurants on a map, collect stars by visiting them, post photo/video reviews, follow food influencers.

---

## Architecture

```
Browser / Mobile App
        │
        ▼
    Nginx :80            ← reverse proxy / API gateway (cloudflared tunnel in prod)
        │
        ├── /api/maps/      → MapsDataService  :3000  (Rust / Axum)
        ├── /api/download/  → DownloadService  :3001  (Rust / Axum)
        ├── /api/auth/      → LoginService     :3002  (Rust / Axum)
        ├── /api/users/     → UserService      :3003  (Rust / Axum)
        ├── /api/posts/     → PostService      :3004  (Rust / Axum)
        ├── /api/feed/      → FeedService      :3005  (Rust / Axum)
        ├── /api/search/    → SearchService    :3006  (Rust / Axum)
        ├── /api/upload/    → UploadService    :3007  (Rust / Axum)
        ├── /api/comments/  → CommentService   :3008  (Rust / Axum)
        ├── /api/likes/     → LikeService      :3009  (Rust / Axum)
        └── /api/stats/     → StatsService     :3010  (Rust / Axum)
```

### MQTT Event Bus (Mosquitto :1883)

| Publisher    | Topic            | Subscribers                |
|--------------|------------------|----------------------------|
| LoginService | user.registered  | SearchService              |
| UserService  | user.updated     | SearchService              |
| PostService  | post.created     | FeedService, StatsService  |
| PostService  | post.deleted     | StatsService               |

### Database Ownership

| Store         | Port  | Owned by                                                      |
|---------------|-------|---------------------------------------------------------------|
| PostgreSQL    | 5432  | LoginService, UserService, UploadService, StatsService, MapsDataService (read) |
| Cassandra     | 9042  | PostService, FeedService, CommentService, LikeService, UserService (follows) |
| Elasticsearch | 9200  | SearchService (read/write via MQTT), Importer (seed)          |

---

## Services

### MapsDataService — `:3000`
Geographic restaurant queries. Returns nearby restaurants with Haversine distance and live experience stats (good_pct from StatsService).

```
GET /api/maps/health
GET /api/maps/restaurants/nearby?lat=&lng=&radius=&limit=
GET /api/maps/restaurants/:id
```

### DownloadService — `:3001`
Serves media files from `./media/` as HLS v3 VOD playlists with full Range-request support for video seeking.

```
GET /api/download/health
GET /api/download/playlist.m3u8[?files=a.mp4,b.jpg]
GET /api/download/files/:name
```

### LoginService — `:3002`
JWT authentication. Issues 15-min access tokens + 30-day refresh tokens. Publishes `user.registered` on signup.

```
POST /api/auth/register    { username, email, password }
POST /api/auth/login       { email, password }
POST /api/auth/refresh     { refresh_token }
POST /api/auth/logout      { refresh_token }
```

### UserService — `:3003`
Profile management, social graph (follow/unfollow), and star collection (honor-system check-in to any Michelin guide entry).

```
GET    /api/users/:id
PATCH  /api/users/me                          🔒
POST   /api/users/:id/follow                  🔒
DELETE /api/users/:id/follow                  🔒
GET    /api/users/:id/followers
GET    /api/users/:id/following
POST   /api/users/me/stars/:restaurant_id     🔒
DELETE /api/users/me/stars/:restaurant_id     🔒
GET    /api/users/:id/stars
```

### PostService — `:3004`
Create and retrieve photo/video posts tagged to restaurants. Publishes MQTT events consumed by FeedService and StatsService.

```
POST   /api/posts                             🔒  { media_id, restaurant_id?, caption, rating }
GET    /api/posts/:id
DELETE /api/posts/:id                         🔒
GET    /api/posts/user/:user_id[?before=&limit=]
GET    /api/posts/restaurant/:restaurant_id[?before=&limit=]
```

`rating` is `"GOOD"` or `"BAD"` (binary Steam-style review).

### FeedService — `:3005`
Personal home feed (fan-out on write via MQTT). Subscribes to `post.created` and writes one row per follower into Cassandra `user_feed`.

```
GET /api/feed[?before=&limit=]    🔒
```

### SearchService — `:3006`
Full-text and faceted search backed by Elasticsearch. Subscribes to `user.registered` and `user.updated` to keep the users index current.

```
GET /api/search/restaurants?q=&city=&country=&cuisine=&award=&lat=&lng=&radius=
GET /api/search/users?q=
```

### UploadService — `:3007`
Accepts multipart file uploads (max 200 MB). Writes to `./media/`, auto-generates video thumbnails via ffmpeg if none provided, stores metadata in PostgreSQL.

```
POST /api/upload    🔒   multipart/form-data  fields: file, thumbnail? (optional)
                         → { media_id, url, thumbnail_url, media_type }
```

### CommentService — `:3008`
Thread-order comments on posts, backed by Cassandra.

```
POST   /api/comments/post/:post_id            🔒  { body }
GET    /api/comments/post/:post_id[?after=&limit=]
DELETE /api/comments/:post_id/:comment_id     🔒
```

### LikeService — `:3009`
Like/unlike posts with atomic Cassandra counters.

```
POST   /api/likes/post/:post_id     🔒
DELETE /api/likes/post/:post_id     🔒
GET    /api/likes/post/:post_id/count
GET    /api/likes/post/:post_id[?limit=]
```

### StatsService — `:3010`
Aggregated good/bad experience per restaurant. Subscribes to MQTT `post.created`/`post.deleted` and upserts `restaurant_stats` in PostgreSQL. MapsDataService JOINs this table.

```
GET /api/stats/restaurants/:id    → { total_posts, good_posts, bad_posts, good_pct }
```

---

## Infrastructure

| Container     | Image                          | Port(s)    | Purpose                               |
|---------------|--------------------------------|------------|---------------------------------------|
| nginx         | nginx:alpine                   | 80, 443    | Reverse proxy / API gateway           |
| postgres      | postgres:16-alpine             | 5432       | Users, restaurants, media, stats      |
| elasticsearch | elasticsearch:8.13.0           | 9200, 9300 | Full-text restaurant + user search    |
| kibana        | kibana:8.13.0                  | 5601       | Elasticsearch UI                      |
| cassandra     | cassandra:5                    | 9042       | Posts, feed, likes, comments, follows |
| mosquitto     | eclipse-mosquitto:2            | 1883, 9001 | MQTT event bus                        |
| importer      | custom                         | —          | Seeds PG + ES from JSONL on boot      |

---

## Quick Start

### Prerequisites
- Docker Desktop
- Rust 1.78+ (local service dev)
- ffmpeg (UploadService thumbnail generation, or use Docker)

### Start infrastructure

```bash
docker compose up -d postgres cassandra elasticsearch mosquitto kibana
# Wait ~60 s for Cassandra to initialise, then:
docker compose up -d cassandra-init importer nginx
```

### Run services locally

```bash
# Copy and fill in the env file for the service you want to run
cp .env.exemple .env

# Example: MapsDataService
cd MapsDataService && cargo run

# Example: LoginService
cd LoginService && cargo run
```

Each service reads env vars from its directory's `.env` or from the shell environment. See `.env.exemple` for all variables.

### Run a service with Docker (once Dockerfile exists)

```bash
docker compose up -d login_service
```

---

## Data Seeding

The `importer` container runs automatically and loads `ressources/all_restaurants.jsonl` into PostgreSQL and Elasticsearch. It also creates the `users` Elasticsearch index. Run once; skips if data already present.

---

## Media Files

Drop files into `./media/` at the project root. They are available immediately via DownloadService without rebuilding.

**Supported:** `jpg jpeg png gif webp bmp mp4 ts m4s mov avi mkv webm`

UploadService writes to `./media/` automatically. DownloadService has read-only access.

---

## Authentication

All `🔒` endpoints require `Authorization: Bearer <jwt>` header.

JWTs are issued by LoginService, validated **independently** in each service using the shared `JWT_SECRET`. There is no round-trip to LoginService on every request.

Token lifetimes: access token 15 min, refresh token 30 days.

---

## Pagination

- **Cassandra-backed endpoints** (feed, posts, comments, likes): cursor-based via `?before=<ISO8601>` (posts) or `?after=<ISO8601>` (comments). Default limit 20, max 100.
- **PostgreSQL-backed endpoints** (star collections, search): offset-based via `?page=&limit=`.
