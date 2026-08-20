-- The original documents_ad trigger omitted `title` from its column list
-- (5 columns, 6 values), so any DELETE on documents failed to prepare.
DROP TRIGGER IF EXISTS documents_ad;

CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, name, title, content, tags) VALUES('delete', old.rowid, old.name, old.title, old.content, old.tags);
END;

-- Rebuild the FTS index from the content table to repair any drift
INSERT INTO documents_fts(documents_fts) VALUES('rebuild');
