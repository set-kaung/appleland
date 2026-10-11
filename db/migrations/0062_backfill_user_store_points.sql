INSERT INTO user_store_points (user_id)
SELECT id
FROM users
ON CONFLICT (user_id) DO NOTHING;
