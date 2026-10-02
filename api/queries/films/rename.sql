UPDATE films
SET title = $3
WHERE id = $1 AND list_id = $2
