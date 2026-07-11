-- Replace single logo_url with up to 3 image URLs (JSON array).
ALTER TABLE proposals ADD COLUMN image_urls TEXT NOT NULL DEFAULT '[]';

UPDATE proposals
SET image_urls = json_array(logo_url)
WHERE logo_url IS NOT NULL AND trim(logo_url) != '';

ALTER TABLE proposals DROP COLUMN logo_url;
