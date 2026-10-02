SELECT
    id,
    list_id,
    starter_id,
    starter_name,
    film_count,
    started_at,
    completed_at
FROM selections
WHERE list_id = any($1) AND completed_at IS NULL
