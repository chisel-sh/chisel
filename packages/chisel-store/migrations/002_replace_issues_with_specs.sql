-- Remove issues tables and triggers
DROP TRIGGER IF EXISTS issues_ai;
DROP TRIGGER IF EXISTS issues_ad;
DROP TRIGGER IF EXISTS issues_au;
DROP TABLE IF EXISTS issues_fts;
DROP TABLE IF EXISTS issues;

-- Create specs table
CREATE TABLE IF NOT EXISTS specs (
    slug TEXT PRIMARY KEY,
    path TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    area TEXT,
    created DATE NOT NULL,
    updated DATE NOT NULL,
    content TEXT NOT NULL
);

-- Create FTS5 virtual table for specs
CREATE VIRTUAL TABLE IF NOT EXISTS specs_fts USING fts5(
    slug UNINDEXED,
    path UNINDEXED,
    title,
    content,
    area,
    content='specs',
    content_rowid='rowid'
);

-- Triggers for specs FTS sync
CREATE TRIGGER IF NOT EXISTS specs_ai AFTER INSERT ON specs BEGIN
    INSERT INTO specs_fts(rowid, title, content, area) VALUES (new.rowid, new.title, new.content, new.area);
END;

CREATE TRIGGER IF NOT EXISTS specs_ad AFTER DELETE ON specs BEGIN
    INSERT INTO specs_fts(specs_fts, rowid, title, content, area) VALUES('delete', old.rowid, old.title, old.content, old.area);
END;

CREATE TRIGGER IF NOT EXISTS specs_au AFTER UPDATE ON specs BEGIN
    INSERT INTO specs_fts(specs_fts, rowid, title, content, area) VALUES('delete', old.rowid, old.title, old.content, old.area);
    INSERT INTO specs_fts(rowid, title, content, area) VALUES (new.rowid, new.title, new.content, new.area);
END;
