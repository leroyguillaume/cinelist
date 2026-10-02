UPDATE participants
SET name = $3
WHERE id = $1 AND list_id = $2
