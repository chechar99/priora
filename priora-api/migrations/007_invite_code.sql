-- Invite codes for shareable neighborhood links (/for/{slug}?invite=…)
ALTER TABLE namespaces ADD COLUMN invite_code TEXT;

UPDATE namespaces
SET invite_code = lower(hex(randomblob(6)))
WHERE invite_code IS NULL OR invite_code = '';

CREATE UNIQUE INDEX IF NOT EXISTS idx_namespaces_invite_code ON namespaces(invite_code);
