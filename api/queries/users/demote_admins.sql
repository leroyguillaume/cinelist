UPDATE users
SET is_admin = FALSE
WHERE is_admin AND lower(username) <> lower($1)
