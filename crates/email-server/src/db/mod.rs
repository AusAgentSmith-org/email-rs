use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};

/// Returns `(pool, has_fts)` — `has_fts` is false if SQLite was built without FTS5.
pub async fn create_pool(database_url: &str) -> anyhow::Result<(SqlitePool, bool)> {
    // Strip sqlite:// prefix to get the file path, then ensure parent dir exists.
    let file_path = database_url
        .strip_prefix("sqlite:///")
        .map(|p| format!("/{p}"))
        .or_else(|| database_url.strip_prefix("sqlite://").map(String::from))
        .unwrap_or_else(|| database_url.to_string());

    if file_path != ":memory:" && !file_path.is_empty() {
        if let Some(parent) = std::path::Path::new(&file_path).parent() {
            if parent != std::path::Path::new("") {
                std::fs::create_dir_all(parent)?;
            }
        }
    }

    let opts: SqliteConnectOptions = database_url
        .parse::<SqliteConnectOptions>()?
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;

    run_migrations(&pool).await?;

    let has_fts = match run_fts_migration(&pool).await {
        Ok(()) => true,
        Err(e) => {
            tracing::warn!("FTS5 unavailable ({e}) — search will use LIKE fallback");
            false
        }
    };

    Ok((pool, has_fts))
}

async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    // Run the initial migration inline to avoid needing DATABASE_URL at compile time.
    let migration_sql = include_str!("migrations/001_initial.sql");

    // Split on statement boundaries and execute each statement.
    // SQLite doesn't support multi-statement execute, so we split manually.
    for statement in migration_sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed).execute(pool).await?;
        }
    }

    // Migration 002: add token_expiry, smtp_host, smtp_port columns.
    // ALTER TABLE fails if the column already exists; ignore "duplicate column name" errors.
    let migration_002 = include_str!("migrations/002_token_expiry.sql");
    for statement in migration_002.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Err(e) = sqlx::query(trimmed).execute(pool).await {
            let msg = e.to_string();
            if msg.contains("duplicate column name") {
                // Column already exists from a previous run — safe to skip.
            } else {
                return Err(e.into());
            }
        }
    }

    let migration_003 = include_str!("migrations/003_settings.sql");
    for statement in migration_003.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Err(e) = sqlx::query(trimmed).execute(pool).await {
            let msg = e.to_string();
            if msg.contains("duplicate column name") {
                // Column already exists — safe to skip.
            } else {
                return Err(e.into());
            }
        }
    }

    let migration_004 = include_str!("migrations/004_webhooks.sql");
    for statement in migration_004.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Err(e) = sqlx::query(trimmed).execute(pool).await {
            let msg = e.to_string();
            if msg.contains("already exists") || msg.contains("duplicate") {
                // Table already exists — safe to skip.
            } else {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

/// Migration 005: FTS5 full-text search index for messages.
/// Written in Rust rather than a SQL file so we can execute each statement
/// individually — SQLite triggers contain embedded semicolons that confuse
/// a naive split-by-';' runner.
async fn run_fts_migration(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
            message_id UNINDEXED,
            subject,
            from_name,
            from_email,
            preview,
            tokenize = 'unicode61'
        )",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TRIGGER IF NOT EXISTS messages_fts_ai
         AFTER INSERT ON messages BEGIN
           INSERT INTO messages_fts(message_id, subject, from_name, from_email, preview)
           VALUES (new.id,
                   coalesce(new.subject,   ''),
                   coalesce(new.from_name, ''),
                   coalesce(new.from_email,''),
                   coalesce(new.preview,   ''));
         END",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TRIGGER IF NOT EXISTS messages_fts_ad
         AFTER DELETE ON messages BEGIN
           INSERT INTO messages_fts(messages_fts, message_id, subject, from_name, from_email, preview)
           VALUES ('delete', old.id,
                   coalesce(old.subject,   ''),
                   coalesce(old.from_name, ''),
                   coalesce(old.from_email,''),
                   coalesce(old.preview,   ''));
         END",
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TRIGGER IF NOT EXISTS messages_fts_au
         AFTER UPDATE OF subject, from_name, from_email, preview ON messages BEGIN
           INSERT INTO messages_fts(messages_fts, message_id, subject, from_name, from_email, preview)
           VALUES ('delete', old.id,
                   coalesce(old.subject,   ''),
                   coalesce(old.from_name, ''),
                   coalesce(old.from_email,''),
                   coalesce(old.preview,   ''));
           INSERT INTO messages_fts(message_id, subject, from_name, from_email, preview)
           VALUES (new.id,
                   coalesce(new.subject,   ''),
                   coalesce(new.from_name, ''),
                   coalesce(new.from_email,''),
                   coalesce(new.preview,   ''));
         END",
    )
    .execute(pool)
    .await?;

    // Backfill any messages not yet in the index (idempotent).
    sqlx::query(
        "INSERT INTO messages_fts(message_id, subject, from_name, from_email, preview)
         SELECT id,
                coalesce(subject,   ''),
                coalesce(from_name, ''),
                coalesce(from_email,''),
                coalesce(preview,   '')
         FROM messages
         WHERE id NOT IN (SELECT message_id FROM messages_fts)",
    )
    .execute(pool)
    .await?;

    Ok(())
}
