ALTER TABLE messages ADD COLUMN snoozed_until TEXT;
CREATE INDEX IF NOT EXISTS idx_messages_snoozed ON messages(snoozed_until) WHERE snoozed_until IS NOT NULL;
