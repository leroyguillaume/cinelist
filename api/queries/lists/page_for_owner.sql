SELECT
    id,
    name,
    created_at
FROM lists
WHERE owner_id = $1
ORDER BY created_at ASC, id ASC
LIMIT $2 OFFSET $3
