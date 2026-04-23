-- ============================================================
-- HackMichelin - PostgreSQL Schema
--
-- Service ownership:
--   LoginService    → users (write), refresh_tokens
--   UserService     → users (counters), user_star_collections
--   UploadService   → media
--   StatsService    → restaurant_stats (write via MQTT events)
--   MapsDataService → restaurants, restaurant_stats (read + JOIN)
--
-- Social layer (posts, likes, comments, feed, follows) → Cassandra
-- Full-text + geo search → Elasticsearch
-- ============================================================

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ─── USERS ────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS users (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    username        VARCHAR(50)  UNIQUE NOT NULL,
    email           VARCHAR(255) UNIQUE NOT NULL,
    password_hash   VARCHAR(255) NOT NULL,
    bio             TEXT,
    avatar_url      TEXT,
    -- Denormalized counters updated by UserService on follow/star events.
    -- Avoids a COUNT(*) on every profile load.
    stars_collected INT          NOT NULL DEFAULT 0,
    followers_count INT          NOT NULL DEFAULT 0,
    following_count INT          NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ  DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  DEFAULT NOW()
);

-- ─── COUNTRIES ────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS countries (
    id   SERIAL       PRIMARY KEY,
    code VARCHAR(10)  UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL
);

-- ─── CITIES ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS cities (
    id          SERIAL       PRIMARY KEY,
    name        VARCHAR(100) NOT NULL,
    region_name VARCHAR(150),
    area_name   VARCHAR(150),
    country_id  INTEGER      NOT NULL REFERENCES countries(id),
    UNIQUE (name, country_id)
);

-- ─── MICHELIN AWARDS ──────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS michelin_awards (
    id             SERIAL      PRIMARY KEY,
    michelin_award VARCHAR(50) UNIQUE NOT NULL
);

-- ─── CUISINE TYPES ────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS type_cuisines (
    code  VARCHAR(50)  PRIMARY KEY,
    label VARCHAR(100)
);

-- ─── PRICE CATEGORIES ─────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS price_categories (
    code  VARCHAR(20)  PRIMARY KEY,
    label VARCHAR(100)
);

-- ─── RESTAURANTS ──────────────────────────────────────────────────────────────
-- Official Michelin guide data. Populated by the importer, read-only after that.
-- Every entry is collectable regardless of award type (starred, green star,
-- bib gourmand, or any other Michelin distinction present in the dataset).
-- city_id / michelin_award_id replace the previous flat string columns.
CREATE TABLE IF NOT EXISTS restaurants (
    id                VARCHAR(50)       PRIMARY KEY,   -- objectID from Michelin dataset
    identifier        VARCHAR(50)       UNIQUE,
    slug              VARCHAR(255)      UNIQUE,
    name              VARCHAR(255)      NOT NULL,
    chef              VARCHAR(255),
    latitude          DOUBLE PRECISION,
    longitude         DOUBLE PRECISION,
    street            TEXT,
    postcode          VARCHAR(20),
    phone             VARCHAR(50),
    website           TEXT,
    short_link        TEXT,
    distinction_score INT,
    guide_year        INT,
    green_star        BOOLEAN           DEFAULT FALSE,
    main_image_url    TEXT,
    main_desc         TEXT,
    online_booking    BOOLEAN           DEFAULT FALSE,
    take_away         BOOLEAN           DEFAULT FALSE,
    delivery          BOOLEAN           DEFAULT FALSE,
    status            VARCHAR(50),
    published_date    TIMESTAMPTZ,
    last_updated      TIMESTAMPTZ,
    michelin_award_id INTEGER           REFERENCES michelin_awards(id),
    city_id           INTEGER           NOT NULL REFERENCES cities(id)
);
CREATE INDEX IF NOT EXISTS restaurants_city_idx        ON restaurants(city_id);
CREATE INDEX IF NOT EXISTS restaurants_award_idx       ON restaurants(michelin_award_id);
CREATE INDEX IF NOT EXISTS restaurants_guide_year_idx  ON restaurants(guide_year);
CREATE INDEX IF NOT EXISTS restaurants_green_star_idx  ON restaurants(green_star) WHERE green_star = TRUE;

-- ─── SERVING (restaurant ↔ cuisine type) ──────────────────────────────────────
CREATE TABLE IF NOT EXISTS serving (
    restaurant_id      VARCHAR(50) REFERENCES restaurants(id) ON DELETE CASCADE,
    type_cuisines_code VARCHAR(50) REFERENCES type_cuisines(code),
    PRIMARY KEY (restaurant_id, type_cuisines_code)
);

-- ─── COSTING (restaurant ↔ price category) ────────────────────────────────────
CREATE TABLE IF NOT EXISTS costing (
    restaurant_id         VARCHAR(50) REFERENCES restaurants(id) ON DELETE CASCADE,
    price_categories_code VARCHAR(20) REFERENCES price_categories(code),
    PRIMARY KEY (restaurant_id, price_categories_code)
);

-- ─── RESTAURANT IMAGES ────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS restaurant_images (
    id            SERIAL      PRIMARY KEY,
    restaurant_id VARCHAR(50) REFERENCES restaurants(id) ON DELETE CASCADE,
    identifier    VARCHAR(50),
    url           TEXT        NOT NULL,
    copyright     TEXT,
    topic         VARCHAR(50),    -- SUJ_PLAT, SUJ_INT, SUJ_ENT, SUJ_EXT …
    "order"       INT         DEFAULT 0,
    UNIQUE (restaurant_id, identifier)
);
CREATE INDEX IF NOT EXISTS restaurant_images_restaurant_idx ON restaurant_images(restaurant_id);

-- ─── MEDIA ────────────────────────────────────────────────────────────────────
-- File-level metadata only. Owned exclusively by UploadService.
-- The social layer (caption, restaurant tag, rating) lives in Cassandra posts,
-- not here. This table is purely about what was stored on disk.
--
-- thumbnail_url: for photos → null (not needed, photo is its own thumbnail).
--                for videos → auto-generated by ffmpeg at upload time.
-- url is /api/download/files/{filename}, computed by UploadService on write.
CREATE TABLE IF NOT EXISTS media (
    id            UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    media_type    VARCHAR(10)  NOT NULL CHECK (media_type IN ('photo', 'video')),
    filename      VARCHAR(255) NOT NULL,
    storage_path  TEXT         NOT NULL,
    url           TEXT         NOT NULL,
    thumbnail_url TEXT,
    mime_type     VARCHAR(100),
    size_bytes    BIGINT,
    width         INT,
    height        INT,
    duration_sec  FLOAT,
    created_at    TIMESTAMPTZ  DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS media_user_idx ON media(user_id);
CREATE INDEX IF NOT EXISTS media_type_idx ON media(media_type);

-- ─── REFRESH TOKENS ───────────────────────────────────────────────────────────
-- LoginService: refresh token store for JWT rotation.
-- JWTs are short-lived (15 min). On login a refresh token (30-day) is issued and
-- its SHA-256 hash stored here. On logout the row is deleted; expired rows can
-- be purged with a periodic DELETE WHERE expires_at < NOW().
CREATE TABLE IF NOT EXISTS refresh_tokens (
    token_hash  VARCHAR(64)  PRIMARY KEY,   -- SHA-256 hex of the raw random token
    user_id     UUID         NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at  TIMESTAMPTZ  NOT NULL,
    created_at  TIMESTAMPTZ  DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS refresh_tokens_user_idx ON refresh_tokens(user_id);

-- ─── USER STAR COLLECTIONS ────────────────────────────────────────────────────
-- UserService: tracks which Michelin guide entries a user has visited.
-- Recognition metadata is snapshotted at collection time so a user's collection
-- stays accurate even if the guide later changes or removes a restaurant's award.
-- PRIMARY KEY prevents collecting the same restaurant twice.
CREATE TABLE IF NOT EXISTS user_star_collections (
    user_id           UUID        REFERENCES users(id)       ON DELETE CASCADE,
    restaurant_id     VARCHAR(50) REFERENCES restaurants(id) ON DELETE CASCADE,
    collected_at      TIMESTAMPTZ DEFAULT NOW(),
    -- Snapshot of the restaurant's Michelin recognition at time of collection
    michelin_award    VARCHAR(50),
    green_star        BOOLEAN     DEFAULT FALSE,
    distinction_score INT,
    PRIMARY KEY (user_id, restaurant_id)
);
CREATE INDEX IF NOT EXISTS star_collections_restaurant_idx ON user_star_collections(restaurant_id);

-- ─── RESTAURANT STATS ─────────────────────────────────────────────────────────
-- StatsService: maintained asynchronously via MQTT post.created / post.deleted.
-- Lives in PostgreSQL so MapsDataService can JOIN it with restaurants in one
-- query and return good_pct inline with the map's nearby results.
--
-- good_pct computation at query time:
--   good_posts::float / NULLIF(total_posts, 0)
CREATE TABLE IF NOT EXISTS restaurant_stats (
    restaurant_id VARCHAR(50) PRIMARY KEY REFERENCES restaurants(id) ON DELETE CASCADE,
    total_posts   INT         NOT NULL DEFAULT 0,
    good_posts    INT         NOT NULL DEFAULT 0,
    bad_posts     INT         NOT NULL DEFAULT 0,
    last_updated  TIMESTAMPTZ DEFAULT NOW()
);
