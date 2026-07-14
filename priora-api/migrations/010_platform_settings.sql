CREATE TABLE platform_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    default_user_role TEXT NOT NULL DEFAULT 'proponent',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO platform_settings (id, default_user_role) VALUES (1, 'proponent');
