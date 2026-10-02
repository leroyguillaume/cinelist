UPDATE invitations
SET used_at = now()
WHERE id = $1
