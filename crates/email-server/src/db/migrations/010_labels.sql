CREATE TABLE IF NOT EXISTS labels (
  id TEXT PRIMARY KEY,
  account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  color TEXT NOT NULL DEFAULT '#6b7280',
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(account_id, name)
);

CREATE TABLE IF NOT EXISTS message_labels (
  id TEXT PRIMARY KEY,
  message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
  label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  UNIQUE(message_id, label_id)
);

CREATE INDEX IF NOT EXISTS idx_labels_account ON labels(account_id);
CREATE INDEX IF NOT EXISTS idx_message_labels_message ON message_labels(message_id);
CREATE INDEX IF NOT EXISTS idx_message_labels_label ON message_labels(label_id);
