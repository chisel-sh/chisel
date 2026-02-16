-- Initial schema for Chisel
-- Includes Docs, Issues, and FTS configuration

-- 1. Documents (Docs)
CREATE TABLE IF NOT EXISTS documents (
    path TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    title TEXT,
    tags TEXT,
    content TEXT NOT NULL,
    created_at DATETIME,
    updated_at DATETIME NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
    path UNINDEXED,
    name,
    title,
    content,
    tags,
    content='documents',
    content_rowid='rowid'
);

-- Triggers for Documents FTS
CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
    INSERT INTO documents_fts(rowid, name, title, content, tags) VALUES (new.rowid, new.name, new.title, new.content, new.tags);
END;

CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, name, content, tags) VALUES('delete', old.rowid, old.name, old.title, old.content, old.tags);
END;

CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
    INSERT INTO documents_fts(documents_fts, rowid, name, title, content, tags) VALUES('delete', old.rowid, old.name, old.title, old.content, old.tags);
    INSERT INTO documents_fts(rowid, name, title, content, tags) VALUES (new.rowid, new.name, new.title, new.content, new.tags);
END;

-- 2. Issues
CREATE TABLE IF NOT EXISTS issues (
    id INTEGER PRIMARY KEY,
    path TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    priority TEXT NOT NULL,
    labels TEXT,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    "order" INTEGER NOT NULL DEFAULT 0,
    external_id TEXT,
    content TEXT NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS issues_fts USING fts5(
    id UNINDEXED,
    path UNINDEXED,
    title,
    content,
    labels,
    content='issues',
    content_rowid='id'
);

-- Triggers for Issues FTS
CREATE TRIGGER IF NOT EXISTS issues_ai AFTER INSERT ON issues BEGIN
    INSERT INTO issues_fts(rowid, title, content, labels) VALUES (new.id, new.title, new.content, new.labels);
END;

CREATE TRIGGER IF NOT EXISTS issues_ad AFTER DELETE ON issues BEGIN
    INSERT INTO issues_fts(issues_fts, rowid, title, content, labels) VALUES('delete', old.id, old.title, old.content, old.labels);
END;

CREATE TRIGGER IF NOT EXISTS issues_au AFTER UPDATE ON issues BEGIN
    INSERT INTO issues_fts(issues_fts, rowid, title, content, labels) VALUES('delete', old.id, old.title, old.content, old.labels);
    INSERT INTO issues_fts(rowid, title, content, labels) VALUES (new.id, new.title, new.content, new.labels);
END;
