SELECT
    id,
    list_id,
    title,
    created_at
FROM films
WHERE list_id = any($1)
ORDER BY list_id ASC, created_at ASC, id ASC
