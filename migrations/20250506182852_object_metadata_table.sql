CREATE TABLE IF NOT EXISTS object_metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bucket_id TEXT,
    object_key TEXT NOT NULL,
    file_name TEXT,
    content_type TEXT,
    content_length INTEGER,
    created_at TEXT NOT NULL,
    FOREIGN KEY (bucket_id) REFERENCES bucket_metadata(id)
);
