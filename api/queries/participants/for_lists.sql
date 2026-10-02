SELECT
    id,
    list_id,
    name
FROM participants
WHERE list_id = any($1)
ORDER BY list_id ASC, turn_order ASC
