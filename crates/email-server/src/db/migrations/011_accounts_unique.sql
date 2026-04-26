-- Remove duplicate accounts caused by re-running OAuth without an account_id.
-- Keep the first-inserted row for each (email, provider_type) pair.
DELETE FROM accounts
WHERE rowid NOT IN (
    SELECT MIN(rowid) FROM accounts
    GROUP BY email, provider_type
);

-- Enforce uniqueness going forward.
CREATE UNIQUE INDEX IF NOT EXISTS idx_accounts_email_provider ON accounts(email, provider_type)
