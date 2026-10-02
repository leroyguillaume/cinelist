INSERT INTO users (id, username, password_hash, is_admin)
VALUES ($1, $2, $3, TRUE)
ON CONFLICT ((lower(username))) DO UPDATE
    SET
        password_hash = excluded.password_hash,
        is_admin = TRUE
