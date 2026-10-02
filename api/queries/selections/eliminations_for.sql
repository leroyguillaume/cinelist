SELECT
    id,
    selection_id,
    film_id,
    film_title,
    participant_id,
    participant_name,
    place
FROM eliminations
WHERE selection_id = any($1)
ORDER BY selection_id ASC, place ASC
