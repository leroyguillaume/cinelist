SELECT
    id,
    list_id,
    starter_id,
    starter_name,
    film_count,
    started_at,
    completed_at
FROM selections
WHERE list_id = $1
ORDER BY started_at DESC
LIMIT $2 OFFSET $3
