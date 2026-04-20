use sqlx::SqlitePool;
use tokio::sync::broadcast;
use tracing::info;

use crate::error::Result;
use crate::imap::sync::ImapSyncEngine;

/// Top-level sync orchestrator: coordinates IMAP, CalDAV and any future
/// sync engines.
pub struct SyncOrchestrator {
    pool: SqlitePool,
    event_tx: broadcast::Sender<String>,
}

impl SyncOrchestrator {
    pub fn new(pool: SqlitePool, event_tx: broadcast::Sender<String>) -> Self {
        Self { pool, event_tx }
    }

    /// Trigger a one-off sync for a specific account.
    pub async fn sync_account(&self, account_id: &str) -> Result<()> {
        info!("manual sync triggered for account {}", account_id);
        let engine = ImapSyncEngine::new(self.pool.clone(), 0);
        engine.sync_account(account_id).await?;
        let _ = self.event_tx.send(format!(
            "{{\"type\":\"sync_complete\",\"accountId\":\"{}\"}}",
            account_id
        ));
        Ok(())
    }

    /// Spawn background sync tasks that run indefinitely (every 300 seconds).
    pub fn spawn_background(pool: SqlitePool, event_tx: broadcast::Sender<String>) {
        let engine = ImapSyncEngine::new(pool, 300);
        tokio::spawn(async move {
            engine.run_with_events(event_tx).await;
        });
    }
}
