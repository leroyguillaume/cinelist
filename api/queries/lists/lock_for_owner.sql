SELECT
    id,
    name,
    created_at
FROM lists
WHERE id = $1 AND owner_id = $2
FOR UPDATE
