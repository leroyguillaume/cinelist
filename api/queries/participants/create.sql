INSERT INTO participants (id, list_id, name, turn_order)
SELECT
    $1 AS id,
    $2 AS list_id,
    $3 AS name,
    coalesce(max(turn_order), 0) + 1 AS turn_order
FROM participants
WHERE list_id = $2
