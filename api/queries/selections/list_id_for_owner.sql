SELECT x.list_id
FROM selections AS x
INNER JOIN lists AS l ON x.list_id = l.id
WHERE x.id = $1 AND l.owner_id = $2
