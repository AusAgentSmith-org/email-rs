use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};
use crate::state::AppState;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct FolderRow {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub full_path: String,
    pub special_use: Option<String>,
    pub unread_count: i64,
    pub total_count: i64,
    pub synced_at: Option<String>,
    pub is_excluded: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchFolder {
    pub is_excluded: Option<bool>,
}

pub async fn patch_folder(
    State(state): State<Arc<AppState>>,
    Path(folder_id): Path<String>,
    Json(body): Json<PatchFolder>,
) -> Result<Json<serde_json::Value>> {
    if let Some(excluded) = body.is_excluded {
        let rows = sqlx::query("UPDATE folders SET is_excluded = ? WHERE id = ?")
            .bind(excluded)
            .bind(&folder_id)
            .execute(&state.pool)
            .await?;
        if rows.rows_affected() == 0 {
            return Err(AppError::NotFound(format!(
                "folder {} not found",
                folder_id
            )));
        }
    }
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn mark_folder_read(
    State(state): State<Arc<AppState>>,
    Path(folder_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let result = sqlx::query("UPDATE messages SET is_read = 1 WHERE folder_id = ?")
        .bind(&folder_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(
        serde_json::json!({ "updated": result.rows_affected() }),
    ))
}

pub async fn list_folders(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<Vec<FolderRow>>> {
    let rows = sqlx::query_as::<_, FolderRow>(
        r#"SELECT id, account_id, name, full_path, special_use,
                  unread_count, total_count, synced_at, is_excluded
           FROM folders
           WHERE account_id = ?
           ORDER BY CASE special_use
               WHEN 'inbox'   THEN 0
               WHEN 'sent'    THEN 1
               WHEN 'drafts'  THEN 2
               WHEN 'archive' THEN 3
               WHEN 'trash'   THEN 4
               WHEN 'spam'    THEN 5
               ELSE 6
           END"#,
    )
    .bind(&account_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}
