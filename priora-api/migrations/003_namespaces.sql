CREATE TABLE namespaces (
    id TEXT PRIMARY KEY NOT NULL,
    slug TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO namespaces (id, slug, name) VALUES
    ('ns-barrio-centro', 'barrio-centro', 'Barrio Centro'),
    ('ns-barrio-norte', 'barrio-norte', 'Barrio Norte');

ALTER TABLE proposals ADD COLUMN namespace_id TEXT REFERENCES namespaces(id);

UPDATE proposals SET namespace_id = 'ns-barrio-centro' WHERE namespace_id IS NULL;

CREATE TABLE user_rankings_new (
    user_id TEXT NOT NULL REFERENCES users(id),
    namespace_id TEXT NOT NULL REFERENCES namespaces(id),
    proposal_id TEXT NOT NULL REFERENCES proposals(id),
    position INTEGER NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, namespace_id, proposal_id)
);

INSERT INTO user_rankings_new (user_id, namespace_id, proposal_id, position, updated_at)
SELECT ur.user_id, p.namespace_id, ur.proposal_id, ur.position, ur.updated_at
FROM user_rankings ur
JOIN proposals p ON p.id = ur.proposal_id;

DROP TABLE user_rankings;
ALTER TABLE user_rankings_new RENAME TO user_rankings;

CREATE INDEX idx_proposals_namespace ON proposals(namespace_id);
CREATE INDEX idx_user_rankings_namespace ON user_rankings(namespace_id, user_id);
