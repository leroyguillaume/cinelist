SELECT
    id,
    list_id,
    starter_id,
    starter_name,
    film_count,
    started_at,
    completed_at
FROM selections
WHERE id = $1 AND list_id = $2
