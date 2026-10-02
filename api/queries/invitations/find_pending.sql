SELECT
    i.id,
    i.user_id,
    u.username,
    i.expires_at
FROM invitations AS i
INNER JOIN users AS u ON i.user_id = u.id
WHERE i.token_hash = $1 AND i.used_at IS NULL AND i.expires_at > now()
FOR UPDATE OF i
