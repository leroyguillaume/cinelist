SELECT
    id,
    username,
    is_admin,
    token_version
FROM users
WHERE id = $1
