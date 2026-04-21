-- ============================================================
-- HackMichelin - Sample Users Seed
-- Passwords: all set to 'Password123!'
-- Hash generated with bcrypt cost 12 via pgcrypto
-- Follows are stored in Cassandra (user_following / user_followers)
-- ============================================================

INSERT INTO users (username, email, password_hash, bio, avatar_url) VALUES
(
    'alice_food',
    'alice@hackmichelin.dev',
    crypt('Password123!', gen_salt('bf', 12)),
    'Fine dining enthusiast. 3-star chaser.',
    'https://api.dicebear.com/8.x/avataaars/svg?seed=alice'
),
(
    'bob_bistro',
    'bob@hackmichelin.dev',
    crypt('Password123!', gen_salt('bf', 12)),
    'Street food lover turned Michelin hunter.',
    'https://api.dicebear.com/8.x/avataaars/svg?seed=bob'
),
(
    'chef_clara',
    'clara@hackmichelin.dev',
    crypt('Password123!', gen_salt('bf', 12)),
    'Professional chef. Reviewer. Green star advocate.',
    'https://api.dicebear.com/8.x/avataaars/svg?seed=clara'
),
(
    'marco_viaggio',
    'marco@hackmichelin.dev',
    crypt('Password123!', gen_salt('bf', 12)),
    'Travelling for taste. Paris → Tokyo → everywhere.',
    'https://api.dicebear.com/8.x/avataaars/svg?seed=marco'
),
(
    'yuki_ramen',
    'yuki@hackmichelin.dev',
    crypt('Password123!', gen_salt('bf', 12)),
    'Tokyo native. Ramen addict. Bib Gourmand fan.',
    'https://api.dicebear.com/8.x/avataaars/svg?seed=yuki'
)
ON CONFLICT (username) DO NOTHING;
