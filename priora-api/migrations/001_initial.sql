CREATE TABLE users (
    id TEXT PRIMARY KEY NOT NULL,
    google_sub TEXT UNIQUE NOT NULL,
    email TEXT NOT NULL,
    name TEXT NOT NULL,
    picture_url TEXT,
    role TEXT NOT NULL DEFAULT 'regular',
    street TEXT,
    floor_apt TEXT,
    city TEXT,
    postal_code TEXT,
    profile_complete INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE proposals (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    logo_url TEXT,
    status TEXT NOT NULL DEFAULT 'activa',
    author_id TEXT NOT NULL REFERENCES users(id),
    tracker_id TEXT REFERENCES users(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE user_rankings (
    user_id TEXT NOT NULL REFERENCES users(id),
    proposal_id TEXT NOT NULL REFERENCES proposals(id),
    position INTEGER NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (user_id, proposal_id)
);

CREATE TABLE comments (
    id TEXT PRIMARY KEY NOT NULL,
    proposal_id TEXT NOT NULL REFERENCES proposals(id) ON DELETE CASCADE,
    author_id TEXT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    edited_at TEXT
);

CREATE INDEX idx_proposals_status ON proposals(status);
CREATE INDEX idx_comments_proposal ON comments(proposal_id);
CREATE INDEX idx_user_rankings_proposal ON user_rankings(proposal_id);
