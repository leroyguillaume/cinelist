SELECT count(*)
FROM lists
WHERE owner_id = $1
