-- Per-space membership and optional approval gate for ranking/comments.
ALTER TABLE namespaces ADD COLUMN require_member_approval INTEGER NOT NULL DEFAULT 0;

CREATE TABLE namespace_members (
    namespace_id TEXT NOT NULL REFERENCES namespaces(id),
    user_id TEXT NOT NULL REFERENCES users(id),
    role TEXT NOT NULL DEFAULT 'regular',
    -- regular | proponent | space_admin
    status TEXT NOT NULL DEFAULT 'active',
    -- pending | active | disabled | rejected
    requested_at TEXT NOT NULL,
    reviewed_at TEXT,
    reviewed_by TEXT REFERENCES users(id),
    PRIMARY KEY (namespace_id, user_id)
);

CREATE INDEX idx_namespace_members_status ON namespace_members(namespace_id, status);
CREATE INDEX idx_namespace_members_user ON namespace_members(user_id);
