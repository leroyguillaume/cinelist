UPDATE selections
SET completed_at = now()
WHERE id = $1
