SELECT
    id,
    list_id,
    title,
    created_at
FROM films
WHERE id = $1 AND list_id = $2
