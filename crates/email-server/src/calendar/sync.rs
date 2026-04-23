use chrono::{Duration, Utc};
use sqlx::SqlitePool;
use tracing::{info, warn};

use crate::auth::oauth2::{OAuthConfig, StoredToken};
use crate::calendar::CalendarService;
use crate::error::{AppError, Result};
use crate::providers::google_calendar::GoogleCalendarProvider;
use crate::providers::CalendarProvider;

// ── Sync a single Gmail account's calendar ────────────────────────────────────

pub async fn sync_gmail_calendar(
    pool: &SqlitePool,
    account_id: &str,
    email: &str,
    token_json: &str,
) -> Result<()> {
    let mut stored: StoredToken = serde_json::from_str(token_json)
        .map_err(|e| AppError::Auth(format!("parse token: {}", e)))?;

    if stored.is_expired() {
        stored = refresh_gmail_token(pool, account_id, &stored).await?;
    }

    let provider = GoogleCalendarProvider::new(
        account_id.to_string(),
        email.to_string(),
        stored.access_token.clone(),
    );

    let now = Utc::now();
    let from = now - Duration::days(7);
    let to = now + Duration::days(60);

    let calendars = match provider.list_calendars().await {
        Ok(c) => c,
        Err(e) => {
            warn!("calendar list_calendars failed for {}: {}", account_id, e);
            return Err(e);
        }
    };

    let service = CalendarService::new(pool.clone());
    let mut synced = 0usize;

    for calendar in &calendars {
        let events = match provider.list_events(&calendar.id, from, to).await {
            Ok(e) => e,
            Err(e) => {
                warn!(
                    "list_events failed for calendar {} (account {}): {}",
                    calendar.id, account_id, e
                );
                continue;
            }
        };

        for event in &events {
            if let Err(e) = service.upsert_event(account_id, event).await {
                warn!("upsert_event failed for {}: {}", event.id, e);
            } else {
                synced += 1;
            }
        }
    }

    info!(
        "calendar sync complete for account {}: {} events across {} calendars",
        account_id,
        synced,
        calendars.len()
    );

    Ok(())
}

// ── Token refresh (mirrors ImapSyncEngine::refresh_token) ────────────────────

async fn refresh_gmail_token(
    pool: &SqlitePool,
    account_id: &str,
    stored: &StoredToken,
) -> Result<StoredToken> {
    let client_id = crate::auth::google_client_id()
        .ok_or_else(|| AppError::Auth("GOOGLE_CLIENT_ID not configured".to_string()))?;
    let client_secret = crate::auth::google_client_secret()
        .ok_or_else(|| AppError::Auth("GOOGLE_CLIENT_SECRET not configured".to_string()))?;

    let refresh = stored.refresh_token.as_deref().ok_or_else(|| {
        AppError::Auth("no refresh token available for calendar sync".to_string())
    })?;

    let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
        .unwrap_or_else(|_| "http://localhost:3000/api/v1/auth/gmail/callback".to_string());

    let oauth = OAuthConfig::gmail(client_id, client_secret, redirect_uri);
    let token_resp = oauth.refresh_token(refresh).await?;

    let mut new_token = StoredToken::from_token_response(token_resp);
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
        .execute(pool)
        .await?;

    Ok(new_token)
}
