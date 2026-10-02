SELECT
    u.id,
    u.username,
    u.is_admin,
    i.expires_at AS invitation_expires_at,
    u.password_hash IS NOT NULL AS has_password,
    (
        SELECT count(*)
        FROM lists AS l
        WHERE l.owner_id = u.id
    ) AS list_count
FROM users AS u
LEFT JOIN LATERAL (
    SELECT inv.expires_at
    FROM invitations AS inv
    WHERE inv.user_id = u.id AND inv.used_at IS NULL
    ORDER BY inv.created_at DESC
    LIMIT 1
) AS i ON TRUE
ORDER BY u.is_admin DESC, lower(u.username) ASC
LIMIT $1 OFFSET $2
