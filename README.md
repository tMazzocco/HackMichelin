# HackMichelin

Michelin-style restaurant review app. Rust/Axum microservices + React 19 frontend.

## Architecture

| Service | Port | Description |
|---|---|---|
| MapsDataService | 3000 | Restaurant geo queries |
| DownloadService | 3001 | File streaming (HLS + static) |
| LoginService | 3002 | Auth (register/login/JWT) |
| UserService | 3003 | Profiles, follows, stars |
| PostService | 3004 | Posts (Cassandra time-series) |
| FeedService | 3005 | Personal feed |
| SearchService | 3006 | Restaurant/user search |
| UploadService | 3007 | Media upload |
| CommentService | 3008 | Comments |
| LikeService | 3009 | Likes |
| StatsService | 3010 | Restaurant stats |

**Databases:** PostgreSQL (restaurants, users, media) · ScyllaDB/Cassandra (posts, feed, comments, likes)

**Frontend:** `mich-front/` — React 19 + TypeScript + Vite + Tailwind. Dev proxy strips `/api/<prefix>` and forwards to the relevant service port.

---

## API Reference

Auth: JWT HS256. Read endpoints are public. Write endpoints require `Authorization: Bearer <token>`.

### LoginService `:3002`

| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/register` | — | Create account |
| POST | `/login` | — | Get access + refresh tokens |
| POST | `/refresh` | — | Rotate refresh token |
| POST | `/logout` | — | Invalidate refresh token |

### UserService `:3003`

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/users/:id` | — | Get profile |
| GET | `/users/:id/followers` | — | List followers |
| GET | `/users/:id/following` | — | List following |
| GET | `/users/:id/stars` | — | List starred restaurants |
| PATCH | `/me` | ✓ | Update own profile |
| POST | `/users/:id/follow` | ✓ | Follow user |
| DELETE | `/users/:id/follow` | ✓ | Unfollow user |
| POST | `/me/stars/:restaurant_id` | ✓ | Star restaurant |
| DELETE | `/me/stars/:restaurant_id` | ✓ | Unstar restaurant |

### PostService `:3004`

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/random` | — | 3 random posts |
| GET | `/:id` | — | Get post |
| GET | `/user/:user_id` | — | User's posts (cursor: `?before=&limit=`) |
| GET | `/restaurant/:restaurant_id` | — | Restaurant's posts (cursor: `?before=&limit=`) |
| POST | `/` | ✓ | Create post |
| DELETE | `/:id` | ✓ | Delete own post |

Post ratings: `"GOOD"` or `"BAD"`. Pagination response: `{ data: Post[], next_before: string|null }`. Exhausted when `data.length < limit`.

### FeedService `:3005`

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/` | ✓ | Personal feed (cursor: `?before=&limit=`) |

### MapsDataService `:3000`

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/restaurants/nearby` | — | `?lat=&lng=&radius=&limit=` |
| GET | `/restaurants/:id` | — | Restaurant detail |

Restaurant fields use `latitude`/`longitude` (not `lat`/`lng`).

### SearchService `:3006`

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/search/restaurants` | — | `?q=` |
| GET | `/api/search/users` | — | `?q=` |

### CommentService `:3008`

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/comments/post/:post_id` | — | List comments |
| POST | `/api/comments/post/:post_id` | ✓ | Create comment |
| DELETE | `/api/comments/:post_id/:comment_id` | ✓ | Delete own comment |

### LikeService `:3009`

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/api/likes/post/:post_id` | — | List likes |
| GET | `/api/likes/post/:post_id/count` | — | Like count |
| POST | `/api/likes/post/:post_id` | ✓ | Like post |
| DELETE | `/api/likes/post/:post_id` | ✓ | Unlike post |

### UploadService `:3007`

| Method | Path | Auth | Description |
|---|---|---|---|
| POST | `/` | ✓ | Upload media, returns `media_id` |

### DownloadService `:3001`

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/playlist.m3u8` | — | HLS playlist |
| GET | `/files/:name` | — | Static file download |

### StatsService `:3010`

| Method | Path | Auth | Description |
|---|---|---|---|
| GET | `/restaurants/:id` | — | Post counts / rating breakdown |

---

## Dev Setup

```bash
# Frontend
cd mich-front
bun install
bun dev
```

Each service reads env vars from `.env`. Key vars: `DATABASE_URL`, `CASSANDRA_NODES`, `HTTP_ADDR`, `JWT_SECRET`.
