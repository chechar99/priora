-- Timeline of proposal lifecycle events (created, status, tracker).
CREATE TABLE proposal_events (
    id TEXT PRIMARY KEY NOT NULL,
    proposal_id TEXT NOT NULL REFERENCES proposals(id) ON DELETE CASCADE,
    event_type TEXT NOT NULL,
    -- created | status_changed | tracker_changed
    actor_id TEXT REFERENCES users(id),
    from_value TEXT,
    to_value TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_proposal_events_proposal ON proposal_events(proposal_id, created_at);

-- Backfill: creation event for every existing proposal.
INSERT INTO proposal_events (id, proposal_id, event_type, actor_id, from_value, to_value, created_at)
SELECT
    lower(hex(randomblob(16))),
    id,
    'created',
    author_id,
    NULL,
    'activa',
    created_at
FROM proposals;

-- Backfill: status change when current status is not the initial activa.
INSERT INTO proposal_events (id, proposal_id, event_type, actor_id, from_value, to_value, created_at)
SELECT
    lower(hex(randomblob(16))),
    id,
    'status_changed',
    NULL,
    'activa',
    status,
    updated_at
FROM proposals
WHERE status != 'activa';

-- Backfill: tracker assignment when a tracker is set.
INSERT INTO proposal_events (id, proposal_id, event_type, actor_id, from_value, to_value, created_at)
SELECT
    lower(hex(randomblob(16))),
    id,
    'tracker_changed',
    NULL,
    NULL,
    tracker_id,
    updated_at
FROM proposals
WHERE tracker_id IS NOT NULL;
