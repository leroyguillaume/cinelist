DELETE FROM invitations
WHERE user_id = $1 AND used_at IS NULL
