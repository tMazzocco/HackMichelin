-- ============================================================
-- HackMichelin - Cassandra Schema
--
-- Service ownership:
--   PostService    → posts, user_posts, restaurant_posts (write)
--   FeedService    → user_feed (write via MQTT fan-out), user_feed (read)
--   LikeService    → post_likes, post_likes_count
--   CommentService → post_comments
--   UserService    → user_following, user_followers
--
-- Design rules:
--   - One table per access pattern (Cassandra has no joins)
--   - All post tables carry a `rating` field ('GOOD' | 'BAD') so the
--     social layer never needs to cross-reference another store for it
--   - posts table uses post_id as sole partition key for O(1) direct lookup
--   - user_posts and restaurant_posts are denormalized write-through copies
--     optimised for their respective list queries
-- ============================================================

CREATE KEYSPACE IF NOT EXISTS hackmichelin
    WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1}
    AND durable_writes = true;

USE hackmichelin;

-- ─── POSTS: canonical lookup by post_id ──────────────────────────────────────
-- Used when a post_id is known and full post data is needed
-- (e.g. CommentService resolving the parent, LikeService resolving the post).
-- Not for list queries — use user_posts or restaurant_posts for those.
CREATE TABLE IF NOT EXISTS posts (
    post_id         uuid        PRIMARY KEY,
    user_id         uuid,
    username        text,
    restaurant_id   text,
    restaurant_name text,
    media_id        uuid,           -- FK to PostgreSQL media.id
    media_type      text,           -- 'photo' | 'video'
    media_url       text,
    thumbnail_url   text,
    caption         text,
    rating          text,           -- 'GOOD' | 'BAD'
    created_at      timestamp
);

-- ─── USER POSTS: timeline per user ───────────────────────────────────────────
-- Used for profile page: "show all posts by user X, newest first".
-- Written in parallel with posts on every PostService create/delete.
CREATE TABLE IF NOT EXISTS user_posts (
    user_id         uuid,
    created_at      timestamp,
    post_id         uuid,
    restaurant_id   text,
    restaurant_name text,
    media_type      text,
    media_url       text,
    thumbnail_url   text,
    caption         text,
    rating          text,
    PRIMARY KEY ((user_id), created_at, post_id)
) WITH CLUSTERING ORDER BY (created_at DESC, post_id ASC);

-- ─── RESTAURANT POSTS: posts tagged to a restaurant ──────────────────────────
-- Used for restaurant detail page and map post overlay.
-- Only written when a post includes a restaurant_id.
-- rating is included here so the frontend can show the good/bad breakdown
-- without a second query to restaurant_stats.
CREATE TABLE IF NOT EXISTS restaurant_posts (
    restaurant_id   text,
    created_at      timestamp,
    post_id         uuid,
    user_id         uuid,
    username        text,
    media_type      text,
    media_url       text,
    thumbnail_url   text,
    caption         text,
    rating          text,
    PRIMARY KEY ((restaurant_id), created_at, post_id)
) WITH CLUSTERING ORDER BY (created_at DESC, post_id ASC);

-- ─── LIKES: dedup guard and display ──────────────────────────────────────────
-- Partition by post_id so "who liked this post?" is a single partition read.
-- Clustering by user_id enables the existence check (has this user liked it?)
-- without a full partition scan.
CREATE TABLE IF NOT EXISTS post_likes (
    post_id  uuid,
    user_id  uuid,
    username text,
    liked_at timestamp,
    PRIMARY KEY ((post_id), user_id)
);

-- ─── LIKE COUNTER ────────────────────────────────────────────────────────────
-- Counter tables must be separate from regular tables in Cassandra.
-- LikeService increments/decrements atomically on like/unlike.
CREATE TABLE IF NOT EXISTS post_likes_count (
    post_id uuid PRIMARY KEY,
    likes   counter
);

-- ─── COMMENTS ────────────────────────────────────────────────────────────────
-- Partition by post_id, cluster ascending so the thread reads top-to-bottom.
-- Pagination cursor: created_at (pass last seen value as ?after=).
CREATE TABLE IF NOT EXISTS post_comments (
    post_id    uuid,
    created_at timestamp,
    comment_id uuid,
    user_id    uuid,
    username   text,
    body       text,
    PRIMARY KEY ((post_id), created_at, comment_id)
) WITH CLUSTERING ORDER BY (created_at ASC, comment_id ASC);

-- ─── USER FEED: personal feed via fan-out on write ───────────────────────────
-- Populated by FeedService on post.created MQTT events.
-- For each follower of the post author, one row is inserted here.
-- 30-day TTL: old feed entries expire automatically without maintenance.
-- Pagination cursor: created_at (pass last seen value as ?before=).
CREATE TABLE IF NOT EXISTS user_feed (
    viewer_id       uuid,
    created_at      timestamp,
    post_id         uuid,
    author_id       uuid,
    author_name     text,
    restaurant_id   text,
    restaurant_name text,
    media_type      text,
    media_url       text,
    thumbnail_url   text,
    caption         text,
    rating          text,
    PRIMARY KEY ((viewer_id), created_at, post_id)
) WITH CLUSTERING ORDER BY (created_at DESC, post_id ASC)
  AND default_time_to_live = 2592000;

-- ─── SOCIAL GRAPH ────────────────────────────────────────────────────────────
-- Two tables for two query directions — Cassandra cannot invert a partition key.
--
-- user_following: "who does user X follow?"
--   → used by FeedService to enumerate whom to fan-out to on post.created
--
-- user_followers: "who follows user X?"
--   → used by UserService to serve the followers list on a profile page
--
-- Both tables are written atomically by UserService on follow/unfollow.
CREATE TABLE IF NOT EXISTS user_following (
    follower_id   uuid,
    followed_id   uuid,
    followed_name text,
    followed_at   timestamp,
    PRIMARY KEY ((follower_id), followed_id)
);

CREATE TABLE IF NOT EXISTS user_followers (
    followed_id   uuid,
    follower_id   uuid,
    follower_name text,
    followed_at   timestamp,
    PRIMARY KEY ((followed_id), follower_id)
);
