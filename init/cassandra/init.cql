-- ============================================================
-- HackMichelin - Cassandra Schema
-- Handles: posts (photo/video), comments, likes, user feed,
--          follows (for feed fan-out), high write throughput
-- ============================================================

CREATE KEYSPACE IF NOT EXISTS hackmichelin
    WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1}
    AND durable_writes = true;

USE hackmichelin;

-- ─── POSTS (photos & videos by users) ────────────────────
-- Partition by user_id, cluster by time DESC → user timeline
CREATE TABLE IF NOT EXISTS posts (
    user_id       uuid,
    created_at    timestamp,
    post_id       uuid,
    media_type    text,        -- 'photo' | 'video'
    media_url     text,
    thumbnail_url text,        -- for videos
    caption       text,
    restaurant_id text,        -- optional Michelin restaurant tag
    PRIMARY KEY ((user_id), created_at, post_id)
) WITH CLUSTERING ORDER BY (created_at DESC)
  AND default_time_to_live = 0;

-- ─── POST LIKES COUNTER ───────────────────────────────────
-- Counter table must be separate from regular table
CREATE TABLE IF NOT EXISTS post_likes_count (
    post_id uuid PRIMARY KEY,
    likes   counter
);

-- ─── WHO LIKED A POST (for dedup check & display) ─────────
-- Partition by post_id → "who liked this post?"
CREATE TABLE IF NOT EXISTS post_likes (
    post_id    uuid,
    user_id    uuid,
    username   text,
    created_at timestamp,
    PRIMARY KEY ((post_id), user_id)
);

-- ─── COMMENTS ─────────────────────────────────────────────
-- Partition by post_id, cluster by time ASC → thread order
CREATE TABLE IF NOT EXISTS comments (
    post_id    uuid,
    created_at timestamp,
    comment_id uuid,
    user_id    uuid,
    username   text,
    body       text,
    PRIMARY KEY ((post_id), created_at, comment_id)
) WITH CLUSTERING ORDER BY (created_at ASC);

-- ─── USER FEED ────────────────────────────────────────────
-- Fan-out on write: when a user posts, insert a row in
-- each follower's feed. Partition by viewer_id → fast feed read.
CREATE TABLE IF NOT EXISTS user_feed (
    viewer_id     uuid,
    created_at    timestamp,
    post_id       uuid,
    author_id     uuid,
    author_name   text,
    media_type    text,
    media_url     text,
    thumbnail_url text,
    caption       text,
    restaurant_id text,
    PRIMARY KEY ((viewer_id), created_at, post_id)
) WITH CLUSTERING ORDER BY (created_at DESC)
  AND default_time_to_live = 2592000;  -- 30-day TTL on feed entries

-- ─── FOLLOWS ────────────────────────────────────────────
-- Two tables for two query patterns:
--   1. user_following: "who does user X follow?" (fan-out writes)
--   2. user_followers: "who follows user X?" (follower list / counts)
CREATE TABLE IF NOT EXISTS user_following (
    follower_id   uuid,
    followed_id   uuid,
    followed_name text,
    created_at    timestamp,
    PRIMARY KEY ((follower_id), followed_id)
);

CREATE TABLE IF NOT EXISTS user_followers (
    followed_id   uuid,
    follower_id   uuid,
    follower_name text,
    created_at    timestamp,
    PRIMARY KEY ((followed_id), follower_id)
);

-- ─── RESTAURANT FEED ──────────────────────────────────────
-- Posts tagged to a specific Michelin restaurant
CREATE TABLE IF NOT EXISTS restaurant_posts (
    restaurant_id text,
    created_at    timestamp,
    post_id       uuid,
    user_id       uuid,
    username      text,
    media_type    text,
    media_url     text,
    thumbnail_url text,
    caption       text,
    PRIMARY KEY ((restaurant_id), created_at, post_id)
) WITH CLUSTERING ORDER BY (created_at DESC);
