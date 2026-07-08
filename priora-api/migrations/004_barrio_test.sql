INSERT OR IGNORE INTO namespaces (id, slug, name) VALUES
    ('ns-barrio-test', 'barrio-test', 'Barrio Test');

UPDATE proposals SET namespace_id = 'ns-barrio-test';

UPDATE user_rankings SET namespace_id = 'ns-barrio-test';

DELETE FROM namespaces WHERE slug IN ('barrio-centro', 'barrio-norte');
