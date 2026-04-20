use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use futures::stream::{self, StreamExt};
use sqlx::SqlitePool;
use tokio::sync::broadcast;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::api::webhooks::fire_webhooks;
use crate::auth::oauth2::{OAuthConfig, StoredToken};
use crate::error::{AppError, Result};
use crate::providers::gmail::GmailProvider;
use crate::providers::MailProvider;

/// Background IMAP sync engine.
/// Periodically polls all configured accounts and updates the local SQLite cache.
pub struct ImapSyncEngine {
    pool: SqlitePool,
    poll_interval_secs: u64,
}

// ── DB row type for account loading ──────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
struct AccountRecord {
    #[allow(dead_code)]
    id: String,
    email: String,
    provider_type: String,
    #[allow(dead_code)]
    auth_type: String,
    oauth_token_json: Option<String>,
    #[allow(dead_code)]
    token_expiry: Option<String>,
    #[allow(dead_code)]
    host: Option<String>,
    #[allow(dead_code)]
    port: Option<i64>,
    #[allow(dead_code)]
    use_ssl: bool,
    sync_days_limit: Option<i64>,
}

#[derive(Debug, sqlx::FromRow)]
struct FolderRecord {
    id: String,
    #[allow(dead_code)]
    name: String,
    full_path: String,
    synced_at: Option<String>,
    is_excluded: bool,
}

// ── ImapSyncEngine ────────────────────────────────────────────────────────────

impl ImapSyncEngine {
    pub fn new(pool: SqlitePool, poll_interval_secs: u64) -> Self {
        Self {
            pool,
            poll_interval_secs,
        }
    }

    /// Sync a single account by id.
    pub async fn sync_account(&self, account_id: &str) -> Result<()> {
        self.sync_account_inner(account_id, None).await
    }

    async fn sync_account_inner(
        &self,
        account_id: &str,
        event_tx: Option<&broadcast::Sender<String>>,
    ) -> Result<()> {
        info!("syncing account {}", account_id);

        // 1. Load account from DB.
        let account = sqlx::query_as::<_, AccountRecord>(
            r#"SELECT id, email, provider_type, auth_type, oauth_token_json,
                      token_expiry, host, port, use_ssl, sync_days_limit
               FROM accounts WHERE id = ?"#,
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("account {} not found", account_id)))?;

        // 2. Build StoredToken from JSON.
        let token_json = account
            .oauth_token_json
            .as_deref()
            .ok_or_else(|| AppError::Auth("account has no OAuth token".to_string()))?;

        let mut stored: StoredToken = serde_json::from_str(token_json)
            .map_err(|e| AppError::Auth(format!("failed to parse token JSON: {}", e)))?;

        // 3. Refresh if expired (or within 5-minute window).
        info!(
            "token expiry for {}: {:?} (expired={})",
            account_id,
            stored.expires_at,
            stored.is_expired()
        );
        if stored.is_expired() {
            info!(
                "access token expired for account {}, refreshing",
                account_id
            );
            stored = self.refresh_token(account_id, &stored).await?;
        }

        match account.provider_type.as_str() {
            "gmail" => {
                let provider = GmailProvider::new(
                    account_id.to_string(),
                    account.email.clone(),
                    stored.access_token.clone(),
                );
                self.sync_provider(account_id, provider, event_tx, account.sync_days_limit)
                    .await?;
            }
            other => {
                warn!(
                    "unsupported provider type '{}' for account {}",
                    other, account_id
                );
            }
        }

        Ok(())
    }

    /// Run the sync loop indefinitely.
    #[allow(dead_code)]
    pub async fn run(self) {
        info!(
            "IMAP sync engine starting, interval={}s",
            self.poll_interval_secs
        );

        loop {
            if let Err(e) = self.sync_all_accounts().await {
                error!("sync cycle error: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(self.poll_interval_secs)).await;
        }
    }

    /// Run the sync loop indefinitely, broadcasting an event after each account sync.
    pub async fn run_with_events(self, event_tx: broadcast::Sender<String>) {
        info!(
            "IMAP sync engine starting (with events), interval={}s",
            self.poll_interval_secs
        );

        loop {
            let account_ids: Vec<String> = match sqlx::query_scalar("SELECT id FROM accounts")
                .fetch_all(&self.pool)
                .await
            {
                Ok(ids) => ids,
                Err(e) => {
                    error!("failed to list accounts: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(self.poll_interval_secs))
                        .await;
                    continue;
                }
            };

            for id in account_ids {
                if let Err(e) = self.sync_account_inner(&id, Some(&event_tx)).await {
                    error!("failed to sync account {}: {}", id, e);
                } else {
                    let _ = event_tx.send(format!(
                        "{{\"type\":\"sync_complete\",\"accountId\":\"{}\"}}",
                        id
                    ));
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(self.poll_interval_secs)).await;
        }
    }

    #[allow(dead_code)]
    async fn sync_all_accounts(&self) -> Result<()> {
        let account_ids: Vec<String> = sqlx::query_scalar("SELECT id FROM accounts")
            .fetch_all(&self.pool)
            .await?;

        for id in account_ids {
            if let Err(e) = self.sync_account(&id).await {
                error!("failed to sync account {}: {}", id, e);
            }
        }
        Ok(())
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Refresh the OAuth token for an account, update DB, and return the new token.
    async fn refresh_token(&self, account_id: &str, stored: &StoredToken) -> Result<StoredToken> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .map_err(|_| AppError::Auth("GOOGLE_CLIENT_ID not set".to_string()))?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .map_err(|_| AppError::Auth("GOOGLE_CLIENT_SECRET not set".to_string()))?;

        let refresh = stored.refresh_token.as_deref().ok_or_else(|| {
            AppError::Auth("no refresh token available for token refresh".to_string())
        })?;

        let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/api/v1/auth/gmail/callback".to_string());

        let oauth = OAuthConfig::gmail(client_id, client_secret, redirect_uri);
        let token_resp = oauth.refresh_token(refresh).await?;

        let mut new_token = StoredToken::from_token_response(token_resp);
        // Preserve existing refresh token if the new response doesn't include one.
        if new_token.refresh_token.is_none() {
            new_token.refresh_token = stored.refresh_token.clone();
        }

        let token_json = serde_json::to_string(&new_token)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("serialize token: {}", e)))?;

        let expiry_str = new_token.expires_at.map(|t| t.to_string());

        sqlx::query("UPDATE accounts SET oauth_token_json = ?, token_expiry = ? WHERE id = ?")
            .bind(&token_json)
            .bind(&expiry_str)
            .bind(account_id)
            .execute(&self.pool)
            .await?;

        Ok(new_token)
    }

    /// Sync folders and messages for a provider, fetching up to 4 folders in parallel.
    async fn sync_provider<P: MailProvider + Clone + Send + Sync + 'static>(
        &self,
        account_id: &str,
        provider: P,
        event_tx: Option<&broadcast::Sender<String>>,
        sync_days_limit: Option<i64>,
    ) -> Result<()> {
        // List folders and upsert (no synced_at — leave NULL for first-time folders
        // so the first fetch uses "ALL" and pulls full history).
        let folders = provider.list_folders().await?;

        for folder in &folders {
            let folder_id = Uuid::new_v4().to_string();
            sqlx::query(
                r#"INSERT INTO folders (id, account_id, name, full_path, special_use)
                   VALUES (?, ?, ?, ?, ?)
                   ON CONFLICT(account_id, full_path) DO UPDATE SET
                       name        = excluded.name,
                       special_use = excluded.special_use"#,
            )
            .bind(&folder_id)
            .bind(account_id)
            .bind(&folder.name)
            .bind(&folder.full_path)
            .bind(&folder.special_use)
            .execute(&self.pool)
            .await?;
        }

        // Reload folder records so we have the actual DB ids; skip excluded folders.
        let db_folders: Vec<FolderRecord> = sqlx::query_as::<_, FolderRecord>(
            "SELECT id, name, full_path, synced_at, is_excluded FROM folders WHERE account_id = ?",
        )
        .bind(account_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .filter(|f| !f.is_excluded)
        .collect();

        let total = db_folders.len();
        info!(
            "syncing {} folders for account {} (4 in parallel)",
            total, account_id
        );

        if let Some(tx) = event_tx {
            let _ = tx.send(format!(
                "{{\"type\":\"sync_start\",\"accountId\":\"{}\",\"folderCount\":{}}}",
                account_id, total
            ));
        }

        let pool = self.pool.clone();
        let account_id = account_id.to_string();
        let done_counter = Arc::new(AtomicUsize::new(0));
        let event_tx_owned: Option<broadcast::Sender<String>> = event_tx.cloned();

        let errors: Vec<String> = stream::iter(db_folders)
            .map(|db_folder| {
                let provider = provider.clone();
                let pool = pool.clone();
                let account_id = account_id.clone();
                let done_counter = done_counter.clone();
                let event_tx = event_tx_owned.clone();
                async move {
                    let folder_name = db_folder.full_path.clone();
                    let result = sync_folder(provider, pool, &account_id, db_folder, sync_days_limit).await;
                    let done = done_counter.fetch_add(1, Ordering::Relaxed) + 1;
                    if let Some(tx) = &event_tx {
                        let _ = tx.send(format!(
                            "{{\"type\":\"sync_folder_done\",\"accountId\":\"{}\",\"folder\":\"{}\",\"done\":{},\"total\":{}}}",
                            account_id, folder_name, done, total
                        ));
                    }
                    result.err().map(|e| e.to_string())
                }
            })
            .buffer_unordered(4)
            .filter_map(|r| async move { r })
            .collect()
            .await;

        for e in &errors {
            error!("folder sync error: {}", e);
        }

        Ok(())
    }
}

/// Fetch and store all messages for a single folder. Runs in parallel with other folders.
async fn sync_folder<P: MailProvider>(
    provider: P,
    pool: SqlitePool,
    account_id: &str,
    db_folder: FolderRecord,
    sync_days_limit: Option<i64>,
) -> crate::error::Result<()> {
    let since = db_folder.synced_at.as_deref().and_then(|s| {
        chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|ndt| ndt.and_utc())
    });

    // If never synced and a days limit is configured, use that as the floor.
    let since = since
        .or_else(|| sync_days_limit.map(|days| chrono::Utc::now() - chrono::Duration::days(days)));

    let messages = provider.fetch_messages(&db_folder.full_path, since).await?;

    let message_count = messages.len() as i64;
    let mut unread_count: i64 = 0;
    let mut new_message_payloads: Vec<serde_json::Value> = Vec::new();

    // Batch all inserts in a single transaction to reduce SQLite write-lock contention
    // when multiple folders are syncing in parallel.
    let mut tx = pool.begin().await?;
    for msg in &messages {
        if !msg.is_read {
            unread_count += 1;
        }
        let msg_id = Uuid::new_v4().to_string();
        let to_json = serde_json::to_string(&msg.to).ok();
        let date_str = msg.date.map(|d| d.to_rfc3339());

        let result = sqlx::query(
            r#"INSERT INTO messages
                   (id, account_id, folder_id, uid, message_id, thread_id,
                    subject, from_name, from_email, to_json, date,
                    is_read, is_flagged, is_draft, has_attachments, preview, synced_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
               ON CONFLICT(message_id) DO UPDATE SET
                   is_read    = excluded.is_read,
                   is_flagged = excluded.is_flagged,
                   synced_at  = excluded.synced_at"#,
        )
        .bind(&msg_id)
        .bind(account_id)
        .bind(&db_folder.id)
        .bind(msg.uid as i64)
        .bind(&msg.message_id)
        .bind(&msg.thread_id)
        .bind(&msg.subject)
        .bind(&msg.from_name)
        .bind(&msg.from_email)
        .bind(&to_json)
        .bind(&date_str)
        .bind(msg.is_read)
        .bind(msg.is_flagged)
        .bind(msg.is_draft)
        .bind(msg.has_attachments)
        .bind(&msg.preview)
        .execute(&mut *tx)
        .await?;

        // Track genuinely new rows for webhook dispatch.
        if result.rows_affected() == 1 {
            new_message_payloads.push(serde_json::json!({
                "subject": msg.subject,
                "from": msg.from_email,
                "fromName": msg.from_name,
                "folder": db_folder.full_path,
                "date": date_str,
            }));
        }
    }
    tx.commit().await?;

    // Fire new_message webhooks for genuinely inserted messages.
    if !new_message_payloads.is_empty() {
        let pool_ref = pool.clone();
        let account_id_str = account_id.to_string();
        tokio::spawn(async move {
            for payload in new_message_payloads {
                fire_webhooks(&pool_ref, &account_id_str, "new_message", payload).await;
            }
        });
    }

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE folder_id = ?")
        .bind(&db_folder.id)
        .fetch_one(&pool)
        .await
        .unwrap_or(message_count);

    let unread: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE folder_id = ? AND is_read = 0")
            .bind(&db_folder.id)
            .fetch_one(&pool)
            .await
            .unwrap_or(unread_count);

    sqlx::query(
        "UPDATE folders SET unread_count = ?, total_count = ?, synced_at = datetime('now') WHERE id = ?",
    )
    .bind(unread)
    .bind(total)
    .bind(&db_folder.id)
    .execute(&pool)
    .await?;

    Ok(())
}
