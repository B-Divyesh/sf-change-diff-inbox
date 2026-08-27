CREATE TABLE IF NOT EXISTS sources (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  url TEXT NOT NULL,
  selector TEXT NOT NULL,
  extract_mode TEXT NOT NULL DEFAULT 'selector',
  threshold REAL NOT NULL DEFAULT 0.03,
  interval_minutes INTEGER NOT NULL DEFAULT 1440,
  enabled INTEGER NOT NULL DEFAULT 1,
  baseline TEXT,
  last_checked TEXT,
  last_status TEXT NOT NULL DEFAULT 'new',
  last_error TEXT,
  next_check TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS changes (
  id TEXT PRIMARY KEY NOT NULL,
  source_id TEXT NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
  previous_text TEXT NOT NULL,
  current_text TEXT NOT NULL,
  change_ratio REAL NOT NULL,
  summary TEXT NOT NULL,
  review_state TEXT NOT NULL DEFAULT 'unread',
  useful INTEGER,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_changes_created ON changes(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_changes_source ON changes(source_id, created_at DESC);

