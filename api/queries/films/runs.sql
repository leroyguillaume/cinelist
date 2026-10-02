SELECT
    s.id AS selection_id,
    s.started_at,
    e.place,
    s.film_count,
    e.participant_name
FROM eliminations AS e
INNER JOIN selections AS s ON e.selection_id = s.id
WHERE e.film_id = $1
ORDER BY s.started_at DESC
LIMIT $2
