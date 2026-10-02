SELECT
    id,
    username,
    is_admin,
    token_version,
    password_hash
FROM users
WHERE lower(username) = lower($1)
