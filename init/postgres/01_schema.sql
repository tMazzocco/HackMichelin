-- ============================================================
-- HackMichelin - PostgreSQL Schema
-- Handles: users, restaurants (structured data), media (URL store)
-- Likes, comments, follows, feed → Cassandra
-- ============================================================

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ─── USERS ────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS users (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username     VARCHAR(50)  UNIQUE NOT NULL,
    email        VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    bio          TEXT,
    avatar_url   TEXT,
    created_at   TIMESTAMPTZ  DEFAULT NOW(),
    updated_at   TIMESTAMPTZ  DEFAULT NOW()
);

-- ─── RESTAURANTS ──────────────────────────────────────────
CREATE TABLE IF NOT EXISTS restaurants (
    id                   VARCHAR(50)  PRIMARY KEY,   -- objectID from Michelin
    identifier           VARCHAR(50)  UNIQUE,
    slug                 VARCHAR(255) UNIQUE,
    name                 VARCHAR(255) NOT NULL,
    chef                 VARCHAR(255),
    lat                  DOUBLE PRECISION,
    lng                  DOUBLE PRECISION,
    city                 VARCHAR(100),
    country_code         VARCHAR(10),
    country_name         VARCHAR(100),
    region_name          VARCHAR(150),
    area_name            VARCHAR(150),
    street               TEXT,
    postcode             VARCHAR(20),
    phone                VARCHAR(50),
    website              TEXT,
    short_link           TEXT,
    michelin_award       VARCHAR(50),
    distinction_score    INT,
    guide_year           INT,
    green_star           BOOLEAN DEFAULT FALSE,
    price_category_code  VARCHAR(20),
    price_category_label VARCHAR(100),
    main_image_url       TEXT,
    main_desc            TEXT,
    online_booking       BOOLEAN DEFAULT FALSE,
    take_away            BOOLEAN DEFAULT FALSE,
    delivery             BOOLEAN DEFAULT FALSE,
    status               VARCHAR(50),
    published_date       TIMESTAMPTZ,
    last_updated         TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS restaurants_city_idx          ON restaurants(city);
CREATE INDEX IF NOT EXISTS restaurants_country_idx       ON restaurants(country_code);
CREATE INDEX IF NOT EXISTS restaurants_michelin_award_idx ON restaurants(michelin_award);
CREATE INDEX IF NOT EXISTS restaurants_guide_year_idx    ON restaurants(guide_year);

-- ─── RESTAURANT IMAGES ────────────────────────────────────
CREATE TABLE IF NOT EXISTS restaurant_images (
    id            SERIAL PRIMARY KEY,
    restaurant_id VARCHAR(50) REFERENCES restaurants(id) ON DELETE CASCADE,
    identifier    VARCHAR(50),
    url           TEXT NOT NULL,
    copyright     TEXT,
    topic         VARCHAR(50),   -- SUJ_PLAT, SUJ_INT, SUJ_ENT, SUJ_EXT …
    "order"       INT DEFAULT 0,
    UNIQUE (restaurant_id, identifier)
);
CREATE INDEX IF NOT EXISTS restaurant_images_restaurant_idx ON restaurant_images(restaurant_id);

-- ─── RESTAURANT CUISINES ──────────────────────────────────
CREATE TABLE IF NOT EXISTS restaurant_cuisines (
    restaurant_id VARCHAR(50) REFERENCES restaurants(id) ON DELETE CASCADE,
    code          VARCHAR(50),
    label         VARCHAR(100),
    slug          VARCHAR(100),
    PRIMARY KEY (restaurant_id, code)
);

-- ─── USER MEDIA ───────────────────────────────────────────
-- Stores metadata for photos/videos uploaded by users.
-- Files are served from /public/ (e.g. /public/uploads/<filename>).
-- media_type: 'photo' | 'video'
CREATE TABLE IF NOT EXISTS media (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    media_type    VARCHAR(10) NOT NULL CHECK (media_type IN ('photo', 'video')),
    url           TEXT NOT NULL,          -- e.g. /public/uploads/abc123.jpg
    thumbnail_url TEXT,                   -- video thumbnail, optional
    filename      VARCHAR(255) NOT NULL,
    mime_type     VARCHAR(100),
    size_bytes    BIGINT,
    width         INT,
    height        INT,
    duration_sec  FLOAT,                  -- video duration, null for photos
    restaurant_id VARCHAR(50) REFERENCES restaurants(id) ON DELETE SET NULL,
    caption       TEXT,
    created_at    TIMESTAMPTZ DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS media_user_idx       ON media(user_id);
CREATE INDEX IF NOT EXISTS media_restaurant_idx ON media(restaurant_id);
CREATE INDEX IF NOT EXISTS media_type_idx       ON media(media_type);

