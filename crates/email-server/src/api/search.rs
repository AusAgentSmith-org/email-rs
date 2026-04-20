use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::api::messages::MessageRow;
use crate::error::Result;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub account_id: Option<String>,
}

// ── FTS5 query helpers ────────────────────────────────────────────────────────

/// Wrap each whitespace-separated word in double quotes for FTS5 exact matching.
/// Double-quotes inside words are escaped by doubling.
fn fts_query(term: &str) -> String {
    term.split_whitespace()
        .map(|w| format!("\"{}\"", w.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Like `fts_query` but the last word gets a `*` prefix suffix for autocomplete.
fn fts_prefix_query(term: &str) -> String {
    let words: Vec<&str> = term.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }
    let mut parts: Vec<String> = words[..words.len() - 1]
        .iter()
        .map(|w| format!("\"{}\"", w.replace('"', "\"\"")))
        .collect();
    let last = words.last().unwrap();
    parts.push(format!("\"{}\"*", last.replace('"', "\"\"")));
    parts.join(" ")
}

// ── Full search ───────────────────────────────────────────────────────────────

pub async fn search_messages(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<MessageRow>>> {
    let term = params.q.trim();
    if term.is_empty() {
        return Ok(Json(vec![]));
    }

    let rows = if state.has_fts {
        search_fts(&state, term, params.account_id.as_deref()).await?
    } else {
        search_like(&state, term, params.account_id.as_deref()).await?
    };

    Ok(Json(rows))
}

async fn search_fts(
    state: &AppState,
    term: &str,
    account_id: Option<&str>,
) -> Result<Vec<MessageRow>> {
    let q = fts_query(term);

    // Column weights: message_id(unindexed)=0, subject=10, from_name=5, from_email=5, preview=2
    let rows = if let Some(aid) = account_id {
        sqlx::query_as::<_, MessageRow>(
            r#"SELECT m.id, m.account_id, m.folder_id, m.uid, m.message_id,
                      m.thread_id, m.subject, m.from_name, m.from_email, m.to_json,
                      m.date, m.is_read, m.is_flagged, m.is_draft, m.has_attachments, m.preview
               FROM messages_fts fts
               JOIN messages m ON m.id = fts.message_id
               WHERE messages_fts MATCH ? AND m.account_id = ?
               ORDER BY bm25(messages_fts, 0, 10, 5, 5, 2)
               LIMIT 50"#,
        )
        .bind(&q)
        .bind(aid)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, MessageRow>(
            r#"SELECT m.id, m.account_id, m.folder_id, m.uid, m.message_id,
                      m.thread_id, m.subject, m.from_name, m.from_email, m.to_json,
                      m.date, m.is_read, m.is_flagged, m.is_draft, m.has_attachments, m.preview
               FROM messages_fts fts
               JOIN messages m ON m.id = fts.message_id
               WHERE messages_fts MATCH ?
               ORDER BY bm25(messages_fts, 0, 10, 5, 5, 2)
               LIMIT 50"#,
        )
        .bind(&q)
        .fetch_all(&state.pool)
        .await?
    };

    Ok(rows)
}

async fn search_like(
    state: &AppState,
    term: &str,
    account_id: Option<&str>,
) -> Result<Vec<MessageRow>> {
    let q_sub = format!("%{term}%");
    let q_word_start = format!("{term} %");
    let q_word_mid = format!("% {term}%");
    let score_expr = r#"CASE
        WHEN lower(m.subject) LIKE lower(?) OR lower(m.subject) LIKE lower(?) THEN 4
        WHEN m.subject LIKE ? THEN 3
        WHEN m.from_name LIKE ? OR m.from_email LIKE ? THEN 2
        WHEN m.preview LIKE ? THEN 1
        ELSE 0 END"#;

    let rows = if let Some(aid) = account_id {
        sqlx::query_as::<_, MessageRow>(&format!(
            r#"SELECT m.id, m.account_id, m.folder_id, m.uid, m.message_id,
                      m.thread_id, m.subject, m.from_name, m.from_email, m.to_json,
                      m.date, m.is_read, m.is_flagged, m.is_draft, m.has_attachments, m.preview
               FROM messages m
               LEFT JOIN message_bodies mb ON mb.message_id = m.id
               WHERE m.account_id = ?
                 AND (m.subject LIKE ? OR m.from_name LIKE ? OR m.from_email LIKE ?
                      OR m.preview LIKE ? OR mb.text_body LIKE ?)
               GROUP BY m.id
               ORDER BY {score_expr} DESC, m.date DESC
               LIMIT 50"#,
        ))
        .bind(aid)
        .bind(&q_sub).bind(&q_sub).bind(&q_sub).bind(&q_sub).bind(&q_sub)
        .bind(&q_word_start).bind(&q_word_mid)
        .bind(&q_sub)
        .bind(&q_sub).bind(&q_sub)
        .bind(&q_sub)
        .fetch_all(&state.pool)
        .await?
    } else {
        sqlx::query_as::<_, MessageRow>(&format!(
            r#"SELECT m.id, m.account_id, m.folder_id, m.uid, m.message_id,
                      m.thread_id, m.subject, m.from_name, m.from_email, m.to_json,
                      m.date, m.is_read, m.is_flagged, m.is_draft, m.has_attachments, m.preview
               FROM messages m
               LEFT JOIN message_bodies mb ON mb.message_id = m.id
               WHERE m.subject LIKE ? OR m.from_name LIKE ? OR m.from_email LIKE ?
                  OR m.preview LIKE ? OR mb.text_body LIKE ?
               GROUP BY m.id
               ORDER BY {score_expr} DESC, m.date DESC
               LIMIT 50"#,
        ))
        .bind(&q_sub).bind(&q_sub).bind(&q_sub).bind(&q_sub).bind(&q_sub)
        .bind(&q_word_start).bind(&q_word_mid)
        .bind(&q_sub)
        .bind(&q_sub).bind(&q_sub)
        .bind(&q_sub)
        .fetch_all(&state.pool)
        .await?
    };

    Ok(rows)
}

// ── Suggest (autocomplete) ────────────────────────────────────────────────────

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SuggestRow {
    pub id: String,
    pub folder_id: String,
    pub subject: Option<String>,
    pub from_name: Option<String>,
    pub from_email: Option<String>,
    pub date: Option<String>,
}

pub async fn suggest_messages(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> Result<Json<Vec<SuggestRow>>> {
    let term = params.q.trim();
    if term.len() < 2 {
        return Ok(Json(vec![]));
    }

    let rows = if state.has_fts {
        let q = fts_prefix_query(term);
        if let Some(aid) = &params.account_id {
            sqlx::query_as::<_, SuggestRow>(
                r#"SELECT m.id, m.folder_id, m.subject, m.from_name, m.from_email, m.date
                   FROM messages_fts fts
                   JOIN messages m ON m.id = fts.message_id
                   WHERE messages_fts MATCH ? AND m.account_id = ?
                   ORDER BY bm25(messages_fts, 0, 10, 5, 5, 2)
                   LIMIT 8"#,
            )
            .bind(&q)
            .bind(aid)
            .fetch_all(&state.pool)
            .await?
        } else {
            sqlx::query_as::<_, SuggestRow>(
                r#"SELECT m.id, m.folder_id, m.subject, m.from_name, m.from_email, m.date
                   FROM messages_fts fts
                   JOIN messages m ON m.id = fts.message_id
                   WHERE messages_fts MATCH ?
                   ORDER BY bm25(messages_fts, 0, 10, 5, 5, 2)
                   LIMIT 8"#,
            )
            .bind(&q)
            .fetch_all(&state.pool)
            .await?
        }
    } else {
        // Prefix LIKE on subject/from only — no leading wildcard so index is usable
        let q_prefix = format!("{term}%");
        let q_sub = format!("%{term}%");
        sqlx::query_as::<_, SuggestRow>(
            r#"SELECT id, folder_id, subject, from_name, from_email, date
               FROM messages
               WHERE subject LIKE ? OR from_name LIKE ? OR from_email LIKE ?
               ORDER BY date DESC
               LIMIT 8"#,
        )
        .bind(&q_prefix)
        .bind(&q_sub)
        .bind(&q_sub)
        .fetch_all(&state.pool)
        .await?
    };

    Ok(Json(rows))
}
