CREATE TABLE categories (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT UNIQUE NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO categories (id, name) VALUES
    ('seguridad', 'Seguridad'),
    ('transito', 'Tránsito'),
    ('movilidad', 'Movilidad'),
    ('recreacion', 'Recreación'),
    ('convivencia', 'Convivencia'),
    ('servicios', 'Servicios');

ALTER TABLE proposals ADD COLUMN category_id TEXT REFERENCES categories(id);

UPDATE proposals SET category_id = 'servicios' WHERE category_id IS NULL;

CREATE INDEX idx_proposals_category ON proposals(category_id);
