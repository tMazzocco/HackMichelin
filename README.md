# HackMichelin

Michelin-style restaurant review app. 11 Rust/Axum microservices behind an Nginx gateway, React 19 frontend.

---

## Architecture

```
Browser / Frontend
      │
      ▼
  Nginx :80  (gateway — routes /api/<service>/* to each upstream)
      │
      ├── /api/maps/       → MapsDataService  :3000
      ├── /api/download/   → DownloadService  :3001
      ├── /api/auth/       → LoginService     :3002
      ├── /api/users/      → UserService      :3003
      ├── /api/posts/      → PostService      :3004
      ├── /api/feed/       → FeedService      :3005
      ├── /api/search/     → SearchService    :3006
      ├── /api/upload/     → UploadService    :3007
      ├── /api/comments/   → CommentService   :3008
      ├── /api/likes/      → LikeService      :3009
      └── /api/stats/      → StatsService     :3010
```

> Nginx strips the gateway prefix before forwarding. `/api/posts/random` → PostService receives `GET /random`.

**Databases**
| Store | Owns |
|---|---|
| PostgreSQL | users, restaurants, media, auth tokens, star collections, stats |
| Cassandra (`hackmichelin` keyspace) | posts, feed, comments, likes, social graph |
| Elasticsearch | full-text + geo restaurant/user search |

**Frontend** — `mich-front/` React 19 + TypeScript + Vite + Tailwind.  
In dev, Vite proxies all `/api/*` to `BACK_URL` (defaults to the Nginx gateway).

---

## API Reference

All paths below are **gateway paths** (what the frontend calls).  
Auth: JWT HS256. Read endpoints are public. Write endpoints require `Authorization: Bearer <token>`.

### Auth — `/api/auth/*` → LoginService :3002

| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/api/auth/register` | — | Create account |
| POST | `/api/auth/login` | — | Returns access token + refresh token |
| POST | `/api/auth/refresh` | — | Rotate refresh token |
| POST | `/api/auth/logout` | — | Invalidate refresh token |

### Users — `/api/users/*` → UserService :3003

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/users/users/:id` | — | Get profile |
| GET | `/api/users/users/:id/followers` | — | List followers |
| GET | `/api/users/users/:id/following` | — | List following |
| GET | `/api/users/users/:id/stars` | — | Starred restaurants |
| PATCH | `/api/users/me` | ✓ | Update own profile |
| POST | `/api/users/users/:id/follow` | ✓ | Follow user |
| DELETE | `/api/users/users/:id/follow` | ✓ | Unfollow user |
| POST | `/api/users/me/stars/:restaurant_id` | ✓ | Star restaurant |
| DELETE | `/api/users/me/stars/:restaurant_id` | ✓ | Unstar restaurant |

### Posts — `/api/posts/*` → PostService :3004

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/posts/random` | — | 3 random posts |
| GET | `/api/posts/:id` | — | Get post by id |
| GET | `/api/posts/user/:user_id` | — | Posts by user (cursor: `?before=&limit=`) |
| GET | `/api/posts/restaurant/:restaurant_id` | — | Posts for restaurant (cursor: `?before=&limit=`) |
| POST | `/api/posts/` | ✓ | Create post |
| DELETE | `/api/posts/:id` | ✓ | Delete own post |

Pagination response: `{ data: Post[], next_before: string|null }`. Exhausted when `data.length < limit`.

### Feed — `/api/feed/*` → FeedService :3005

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/feed/` | ✓ | Personal feed (cursor: `?before=&limit=`) |

### Maps — `/api/maps/*` → MapsDataService :3000

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/maps/restaurants/nearby` | — | `?lat=&lng=&radius=&limit=` |
| GET | `/api/maps/restaurants/:id` | — | Restaurant detail |

> Restaurant geo fields are named `latitude` / `longitude`.

### Search — `/api/search/*` → SearchService :3006

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/search/api/search/restaurants` | — | `?q=` |
| GET | `/api/search/api/search/users` | — | `?q=` |

### Upload — `/api/upload/*` → UploadService :3007

| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/api/upload/` | ✓ | Upload photo or video. Returns `{ media_id }`. Max 200 MB. |

### Download — `/api/download/*` → DownloadService :3001

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/download/files/:filename` | — | Serve media file (supports Range) |
| GET | `/api/download/playlist.m3u8` | — | HLS playlist (`?files=a.mp4,b.jpg,...`) |

### Comments — `/api/comments/*` → CommentService :3008

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/comments/api/comments/post/:post_id` | — | List comments |
| POST | `/api/comments/api/comments/post/:post_id` | ✓ | Create comment |
| DELETE | `/api/comments/api/comments/:post_id/:comment_id` | ✓ | Delete own comment |

### Likes — `/api/likes/*` → LikeService :3009

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/likes/api/likes/post/:post_id` | — | List likes |
| GET | `/api/likes/api/likes/post/:post_id/count` | — | Like count |
| POST | `/api/likes/api/likes/post/:post_id` | ✓ | Like post |
| DELETE | `/api/likes/api/likes/post/:post_id` | ✓ | Unlike post |

### Stats — `/api/stats/*` → StatsService :3010

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/stats/restaurants/:id` | — | Post counts + rating breakdown |

---

## Dev Setup

```bash
# Start infrastructure (postgres, cassandra, nginx, mqtt, elasticsearch)
docker compose up -d postgres cassandra mosquitto elasticsearch nginx

# Apply Cassandra schema
docker compose run --rm cassandra-init

# Import restaurant data
docker compose run --rm importer

# Seed demo posts (3 videos + user)
bash init/import/seed_posts.sh
bash init/import/seed_thumbnails.sh

# Frontend
cd mich-front
bun install
bun dev        # proxies /api/* to gateway at BACK_URL
```

### Environment Variables

Each service reads from `.env` or environment. Key vars:

| Var | Used by | Default |
|---|---|---|
| `DATABASE_URL` | All PG services | `postgresql://admin:changeme@postgres:5432/hackmichelin` |
| `CASSANDRA_NODES` | PostService, FeedService, LikeService, CommentService | `cassandra` |
| `JWT_SECRET` | LoginService, all protected services | `change-me` |
| `HTTP_ADDR` | All services | `0.0.0.0:<port>` |
| `MEDIA_DIR` | UploadService, DownloadService | `/media` |
| `DOWNLOAD_BASE_URL` | UploadService | `/api/download/files` |

---

## Seed Data

Demo user and 3 video posts created by `init/import/seed_posts.sh`:

| Field | Value |
|---|---|
| Username | `michelin_reviewer` |
| Password | `password123` |
| User ID | `11111111-1111-1111-1111-111111111111` |

| Post | Restaurant | Restaurant ID | Video file |
|---|---|---|---|
| Toulouse | Dozo | 134818 | `Bonne adresse à Toulouse - Dozo.mp4` |
| Agen | *(resolved from DB)* | 133250 | `Restaurante 3 estrelas Michelin em Agen.mp4` |
| Tokyo | Yaumay | 123358 | `Sustainable Thai Cuisine at Baan Tepa.mp4` |

Thumbnails: `minia-dozo.png`, `minia-paris.png`, `minia-tokyo.png` — applied by `seed_thumbnails.sh`.
