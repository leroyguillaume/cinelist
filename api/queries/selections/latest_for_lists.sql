SELECT DISTINCT ON (list_id)
    id,
    list_id,
    starter_id,
    starter_name,
    film_count,
    started_at,
    completed_at
FROM selections
WHERE list_id = any($1)
ORDER BY list_id ASC, started_at DESC
