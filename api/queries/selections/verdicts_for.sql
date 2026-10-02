SELECT
    selection_id,
    film_title,
    place,
    outcome,
    created_at
FROM verdicts
WHERE selection_id = any($1)
ORDER BY selection_id ASC, place ASC
