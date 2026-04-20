use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use chrono;

// ── Bulk action ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkAction {
    Archive,
    Delete,
    MarkRead,
    MarkUnread,
}

#[derive(Debug, Deserialize)]
pub struct BulkRequest {
    pub ids: Vec<String>,
    pub action: BulkAction,
}

pub async fn bulk_messages(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BulkRequest>,
) -> crate::error::Result<Json<serde_json::Value>> {
    let mut processed = 0usize;

    for id in &req.ids {
        match req.action {
            BulkAction::MarkRead => {
                sqlx::query("UPDATE messages SET is_read = 1 WHERE id = ?")
                    .bind(id)
                    .execute(&state.pool)
                    .await?;

                if let Ok(Some((info, provider))) = load_provider(&state, id).await {
                    let uid = info.uid as u32;
                    let folder = info.full_path.clone();
                    let p = provider.clone();
                    tokio::spawn(async move {
                        if let Err(e) = p.mark_seen(&folder, uid).await {
                            tracing::warn!("bulk mark_seen failed: {}", e);
                        }
                    });
                }
            }
            BulkAction::MarkUnread => {
                sqlx::query("UPDATE messages SET is_read = 0 WHERE id = ?")
                    .bind(id)
                    .execute(&state.pool)
                    .await?;

                if let Ok(Some((info, provider))) = load_provider(&state, id).await {
                    let uid = info.uid as u32;
                    let folder = info.full_path.clone();
                    let p = provider.clone();
                    tokio::spawn(async move {
                        if let Err(e) = p.mark_unseen(&folder, uid).await {
                            tracing::warn!("bulk mark_unseen failed: {}", e);
                        }
                    });
                }
            }
            BulkAction::Delete => {
                if let Ok(Some((info, provider))) = load_provider(&state, id).await {
                    let uid = info.uid as u32;
                    let folder = info.full_path.clone();
                    tokio::spawn(async move {
                        if let Err(e) = provider.delete_message(&folder, uid).await {
                            tracing::warn!("bulk IMAP delete failed: {}", e);
                        }
                    });
                }
                sqlx::query("DELETE FROM messages WHERE id = ?")
                    .bind(id)
                    .execute(&state.pool)
                    .await?;
            }
            BulkAction::Archive => {
                let archive_folder: Option<String> = sqlx::query_scalar(
                    r#"SELECT f.full_path FROM folders f
                       JOIN messages m ON m.account_id = f.account_id
                       WHERE m.id = ? AND f.special_use = 'archive'
                       LIMIT 1"#,
                )
                .bind(id)
                .fetch_optional(&state.pool)
                .await?;

                let archive_full_path =
                    archive_folder.unwrap_or_else(|| "[Gmail]/All Mail".to_string());

                let archive_folder_id: Option<String> = sqlx::query_scalar(
                    r#"SELECT f.id FROM folders f
                       JOIN messages m ON m.account_id = f.account_id
                       WHERE m.id = ? AND f.special_use = 'archive'
                       LIMIT 1"#,
                )
                .bind(id)
                .fetch_optional(&state.pool)
                .await?;

                if let Ok(Some((info, provider))) = load_provider(&state, id).await {
                    let uid = info.uid as u32;
                    let src = info.full_path.clone();
                    let dest = archive_full_path.clone();
                    tokio::spawn(async move {
                        if let Err(e) = provider.move_message(&src, uid, &dest).await {
                            tracing::warn!("bulk IMAP archive failed: {}", e);
                        }
                    });
                }

                if let Some(folder_id) = archive_folder_id {
                    sqlx::query("UPDATE messages SET folder_id = ? WHERE id = ?")
                        .bind(&folder_id)
                        .bind(id)
                        .execute(&state.pool)
                        .await?;
                } else {
                    sqlx::query("DELETE FROM messages WHERE id = ?")
                        .bind(id)
                        .execute(&state.pool)
                        .await?;
                }
            }
        }
        processed += 1;
    }

    Ok(Json(serde_json::json!({ "processed": processed })))
}

use crate::auth::oauth2::{OAuthConfig, StoredToken};
use crate::error::{AppError, Result};
use crate::providers::gmail::GmailProvider;
use crate::providers::MailProvider;
use crate::state::AppState;

#[derive(Debug, sqlx::FromRow)]
struct MessageActionInfo {
    uid: i64,
    full_path: String,
    email: String,
    provider_type: String,
    oauth_token_json: Option<String>,
}

async fn load_provider(
    state: &AppState,
    message_id: &str,
) -> Result<Option<(MessageActionInfo, GmailProvider)>> {
    let info = sqlx::query_as::<_, MessageActionInfo>(
        r#"SELECT m.uid, f.full_path, a.email, a.provider_type, a.oauth_token_json
           FROM messages m
           JOIN folders f ON m.folder_id = f.id
           JOIN accounts a ON m.account_id = a.id
           WHERE m.id = ?"#,
    )
    .bind(message_id)
    .fetch_optional(&state.pool)
    .await?;

    let Some(info) = info else { return Ok(None) };
    if info.provider_type != "gmail" {
        return Ok(None);
    }

    let token_json = info
        .oauth_token_json
        .as_deref()
        .ok_or_else(|| AppError::Auth("account has no OAuth token".to_string()))?;
    let mut stored: StoredToken = serde_json::from_str(token_json)
        .map_err(|e| AppError::Auth(format!("parse token: {}", e)))?;
    if stored.is_expired() {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .map_err(|_| AppError::Auth("GOOGLE_CLIENT_ID not set".to_string()))?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .map_err(|_| AppError::Auth("GOOGLE_CLIENT_SECRET not set".to_string()))?;
        let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI")
            .unwrap_or_else(|_| "http://localhost:3000/api/v1/auth/gmail/callback".to_string());
        let oauth = OAuthConfig::gmail(client_id, client_secret, redirect_uri);
        let refresh = stored
            .refresh_token
            .as_deref()
            .ok_or_else(|| AppError::Auth("no refresh token".to_string()))?;
        stored = StoredToken::from_token_response(oauth.refresh_token(refresh).await?);
    }

    let account_id: String = sqlx::query_scalar("SELECT account_id FROM messages WHERE id = ?")
        .bind(message_id)
        .fetch_one(&state.pool)
        .await?;

    let provider = GmailProvider::new(account_id, info.email.clone(), stored.access_token.clone());
    Ok(Some((info, provider)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchMessageRequest {
    pub is_read: Option<bool>,
    pub is_flagged: Option<bool>,
}

pub async fn patch_message(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
    Json(req): Json<PatchMessageRequest>,
) -> Result<Json<serde_json::Value>> {
    if let Some(is_read) = req.is_read {
        sqlx::query("UPDATE messages SET is_read = ? WHERE id = ?")
            .bind(is_read)
            .bind(&message_id)
            .execute(&state.pool)
            .await?;

        if let Ok(Some((info, provider))) = load_provider(&state, &message_id).await {
            let uid = info.uid as u32;
            let folder = info.full_path.clone();
            let p = provider.clone();
            tokio::spawn(async move {
                let result = if is_read {
                    p.mark_seen(&folder, uid).await
                } else {
                    p.mark_unseen(&folder, uid).await
                };
                if let Err(e) = result {
                    tracing::warn!("mark_seen/unseen failed: {}", e);
                }
            });
        }
    }

    if let Some(is_flagged) = req.is_flagged {
        sqlx::query("UPDATE messages SET is_flagged = ? WHERE id = ?")
            .bind(is_flagged)
            .bind(&message_id)
            .execute(&state.pool)
            .await?;

        if let Ok(Some((info, provider))) = load_provider(&state, &message_id).await {
            let uid = info.uid as u32;
            let folder = info.full_path.clone();
            tokio::spawn(async move {
                if let Err(e) = provider.set_flagged(&folder, uid, is_flagged).await {
                    tracing::warn!("set_flagged failed: {}", e);
                }
            });
        }
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn delete_message(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    // Fire IMAP delete in background.
    if let Ok(Some((info, provider))) = load_provider(&state, &message_id).await {
        let uid = info.uid as u32;
        let folder = info.full_path.clone();
        tokio::spawn(async move {
            if let Err(e) = provider.delete_message(&folder, uid).await {
                tracing::warn!("IMAP delete_message failed: {}", e);
            }
        });
    }

    // Remove from local DB immediately.
    sqlx::query("DELETE FROM messages WHERE id = ?")
        .bind(&message_id)
        .execute(&state.pool)
        .await?;

    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

pub async fn archive_message(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    // Find the archive folder for this message's account.
    let archive_folder: Option<String> = sqlx::query_scalar(
        r#"SELECT f.full_path FROM folders f
           JOIN messages m ON m.account_id = f.account_id
           WHERE m.id = ? AND f.special_use = 'archive'
           LIMIT 1"#,
    )
    .bind(&message_id)
    .fetch_optional(&state.pool)
    .await?;

    let archive_full_path = archive_folder.unwrap_or_else(|| "[Gmail]/All Mail".to_string());

    // Find the new folder_id for the archive folder in the DB.
    let archive_folder_id: Option<String> = sqlx::query_scalar(
        r#"SELECT f.id FROM folders f
           JOIN messages m ON m.account_id = f.account_id
           WHERE m.id = ? AND f.special_use = 'archive'
           LIMIT 1"#,
    )
    .bind(&message_id)
    .fetch_optional(&state.pool)
    .await?;

    if let Ok(Some((info, provider))) = load_provider(&state, &message_id).await {
        let uid = info.uid as u32;
        let src_folder = info.full_path.clone();
        let dest = archive_full_path.clone();
        tokio::spawn(async move {
            if let Err(e) = provider.move_message(&src_folder, uid, &dest).await {
                tracing::warn!("IMAP archive failed: {}", e);
            }
        });
    }

    if let Some(folder_id) = archive_folder_id {
        sqlx::query("UPDATE messages SET folder_id = ? WHERE id = ?")
            .bind(&folder_id)
            .bind(&message_id)
            .execute(&state.pool)
            .await?;
    } else {
        sqlx::query("DELETE FROM messages WHERE id = ?")
            .bind(&message_id)
            .execute(&state.pool)
            .await?;
    }

    Ok(Json(serde_json::json!({ "status": "archived" })))
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MessageRow {
    pub id: String,
    pub account_id: String,
    pub folder_id: String,
    pub uid: i64,
    pub message_id: Option<String>,
    pub thread_id: Option<String>,
    pub subject: Option<String>,
    pub from_name: Option<String>,
    pub from_email: Option<String>,
    pub to_json: Option<String>,
    pub date: Option<String>,
    pub is_read: bool,
    pub is_flagged: bool,
    pub is_draft: bool,
    pub has_attachments: bool,
    pub preview: Option<String>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MessageBodyRow {
    pub message_id: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
    pub raw_headers: Option<String>,
    pub fetched_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ListMessagesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub unread_only: Option<bool>,
}

pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Path(folder_id): Path<String>,
    Query(q): Query<ListMessagesQuery>,
) -> Result<Json<Vec<MessageRow>>> {
    let page = q.page.unwrap_or(1).max(1);
    let per_page = q.per_page.unwrap_or(50).clamp(1, 200);
    let offset = (page - 1) * per_page;
    let unread_only = q.unread_only.unwrap_or(false);

    let sql = if unread_only {
        r#"SELECT id, account_id, folder_id, uid, message_id, thread_id,
                  subject, from_name, from_email, to_json, date,
                  is_read, is_flagged, is_draft, has_attachments, preview
           FROM messages
           WHERE folder_id = ? AND is_read = 0
           ORDER BY date DESC
           LIMIT ? OFFSET ?"#
    } else {
        r#"SELECT id, account_id, folder_id, uid, message_id, thread_id,
                  subject, from_name, from_email, to_json, date,
                  is_read, is_flagged, is_draft, has_attachments, preview
           FROM messages
           WHERE folder_id = ?
           ORDER BY date DESC
           LIMIT ? OFFSET ?"#
    };

    let rows = sqlx::query_as::<_, MessageRow>(sql)
        .bind(&folder_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(&state.pool)
        .await?;

    Ok(Json(rows))
}

/// Row type for the join query used during lazy body fetch.
#[derive(Debug, sqlx::FromRow)]
struct MessageFetchInfo {
    uid: i64,
    full_path: String,
    email: String,
    provider_type: String,
    oauth_token_json: Option<String>,
}

pub async fn get_message(
    State(state): State<Arc<AppState>>,
    Path(message_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let msg = sqlx::query_as::<_, MessageRow>(
        r#"SELECT id, account_id, folder_id, uid, message_id, thread_id,
                  subject, from_name, from_email, to_json, date,
                  is_read, is_flagged, is_draft, has_attachments, preview
           FROM messages WHERE id = ?"#,
    )
    .bind(&message_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("message {} not found", message_id)))?;

    // Check body cache.
    let mut body = sqlx::query_as::<_, MessageBodyRow>(
        "SELECT message_id, html_body, text_body, raw_headers, fetched_at FROM message_bodies WHERE message_id = ?",
    )
    .bind(&message_id)
    .fetch_optional(&state.pool)
    .await?;

    // Always load provider info — needed for both lazy body fetch and mark_seen.
    let fetch_info = sqlx::query_as::<_, MessageFetchInfo>(
        r#"SELECT m.uid, f.full_path, a.email, a.provider_type, a.oauth_token_json
           FROM messages m
           JOIN folders f ON m.folder_id = f.id
           JOIN accounts a ON m.account_id = a.id
           WHERE m.id = ?"#,
    )
    .bind(&message_id)
    .fetch_optional(&state.pool)
    .await?;

    // Build provider from fetch_info (shared by body fetch + mark_seen).
    let provider = if let Some(ref info) = fetch_info {
        if info.provider_type == "gmail" {
            let token_json = info
                .oauth_token_json
                .as_deref()
                .ok_or_else(|| AppError::Auth("account has no OAuth token".to_string()))?;

            let mut stored: StoredToken = serde_json::from_str(token_json)
                .map_err(|e| AppError::Auth(format!("parse token: {}", e)))?;

            if stored.is_expired() {
                let client_id = std::env::var("GOOGLE_CLIENT_ID")
                    .map_err(|_| AppError::Auth("GOOGLE_CLIENT_ID not set".to_string()))?;
                let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
                    .map_err(|_| AppError::Auth("GOOGLE_CLIENT_SECRET not set".to_string()))?;
                let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI").unwrap_or_else(|_| {
                    "http://localhost:3000/api/v1/auth/gmail/callback".to_string()
                });
                let oauth = OAuthConfig::gmail(client_id, client_secret, redirect_uri);
                let refresh = stored
                    .refresh_token
                    .as_deref()
                    .ok_or_else(|| AppError::Auth("no refresh token".to_string()))?;
                let new_resp = oauth.refresh_token(refresh).await?;
                stored = StoredToken::from_token_response(new_resp);
            }

            Some(GmailProvider::new(
                msg.account_id.clone(),
                info.email.clone(),
                stored.access_token.clone(),
            ))
        } else {
            None
        }
    } else {
        None
    };

    // Lazy-fetch body from IMAP on cache miss.
    if body.is_none() {
        if let (Some(ref info), Some(ref p)) = (&fetch_info, &provider) {
            info!("body cache miss for {}, fetching from IMAP", message_id);
            match p.fetch_message_body(&info.full_path, info.uid as u32).await {
                Ok(fetched) => {
                    sqlx::query(
                        r#"INSERT INTO message_bodies (message_id, html_body, text_body, raw_headers)
                           VALUES (?, ?, ?, ?)
                           ON CONFLICT(message_id) DO UPDATE SET
                               html_body  = excluded.html_body,
                               text_body  = excluded.text_body,
                               raw_headers = excluded.raw_headers,
                               fetched_at = datetime('now')"#,
                    )
                    .bind(&message_id)
                    .bind(&fetched.html_body)
                    .bind(&fetched.text_body)
                    .bind(&fetched.raw_headers)
                    .execute(&state.pool)
                    .await?;

                    body = Some(MessageBodyRow {
                        message_id: message_id.clone(),
                        html_body: fetched.html_body,
                        text_body: fetched.text_body,
                        raw_headers: fetched.raw_headers,
                        fetched_at: chrono::Utc::now().to_rfc3339(),
                    });
                }
                Err(e) => {
                    tracing::warn!("failed to fetch IMAP body for {}: {}", message_id, e);
                }
            }
        }
    }

    // Fire-and-forget: set \Seen on the IMAP server (doesn't block the response).
    if !msg.is_read {
        if let (Some(info), Some(p)) = (fetch_info, provider) {
            tokio::spawn(async move {
                if let Err(e) = p.mark_seen(&info.full_path, info.uid as u32).await {
                    tracing::warn!("mark_seen failed: {}", e);
                }
            });
        }
    }

    // Mark as read in local DB.
    sqlx::query("UPDATE messages SET is_read = 1 WHERE id = ?")
        .bind(&message_id)
        .execute(&state.pool)
        .await?;

    let result = serde_json::json!({
        "id": msg.id,
        "accountId": msg.account_id,
        "folderId": msg.folder_id,
        "uid": msg.uid,
        "messageId": msg.message_id,
        "threadId": msg.thread_id,
        "subject": msg.subject,
        "fromName": msg.from_name,
        "fromEmail": msg.from_email,
        "to": msg.to_json,
        "date": msg.date,
        "isRead": true,
        "isFlagged": msg.is_flagged,
        "isDraft": msg.is_draft,
        "hasAttachments": msg.has_attachments,
        "preview": msg.preview,
        "body": body.map(|b| serde_json::json!({
            "htmlBody": b.html_body,
            "textBody": b.text_body,
        })),
    });

    Ok(Json(result))
}
