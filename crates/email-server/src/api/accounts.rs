use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::state::AppState;
use crate::sync::SyncOrchestrator;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AccountRow {
    pub id: String,
    pub name: String,
    pub email: String,
    pub provider_type: String,
    pub auth_type: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAccount {
    pub name: String,
    pub email: String,
    pub provider_type: String,
    pub auth_type: String,
    pub oauth_token_json: Option<String>,
    pub host: Option<String>,
    pub port: Option<i64>,
    pub use_ssl: Option<bool>,
}

pub async fn list_accounts(State(state): State<Arc<AppState>>) -> Result<Json<Vec<AccountRow>>> {
    let rows = sqlx::query_as::<_, AccountRow>(
        "SELECT id, name, email, provider_type, auth_type, created_at FROM accounts ORDER BY created_at"
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rows))
}

pub async fn create_account(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateAccount>,
) -> Result<Json<AccountRow>> {
    let id = Uuid::new_v4().to_string();
    let use_ssl = body.use_ssl.unwrap_or(true);

    sqlx::query(
        r#"INSERT INTO accounts (id, name, email, provider_type, auth_type, oauth_token_json, host, port, use_ssl)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(&body.name)
    .bind(&body.email)
    .bind(&body.provider_type)
    .bind(&body.auth_type)
    .bind(&body.oauth_token_json)
    .bind(&body.host)
    .bind(body.port)
    .bind(use_ssl)
    .execute(&state.pool)
    .await?;

    let row = sqlx::query_as::<_, AccountRow>(
        "SELECT id, name, email, provider_type, auth_type, created_at FROM accounts WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(row))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccount {
    pub name: Option<String>,
    pub sync_days_limit: Option<i64>,
    pub signature: Option<String>,
}

pub async fn update_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateAccount>,
) -> Result<Json<AccountRow>> {
    if let Some(ref name) = body.name {
        sqlx::query("UPDATE accounts SET name = ? WHERE id = ?")
            .bind(name)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }
    if let Some(days) = body.sync_days_limit {
        sqlx::query("UPDATE accounts SET sync_days_limit = ? WHERE id = ?")
            .bind(days)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }
    if let Some(ref sig) = body.signature {
        sqlx::query("UPDATE accounts SET signature = ? WHERE id = ?")
            .bind(sig)
            .bind(&id)
            .execute(&state.pool)
            .await?;
    }
    let row = sqlx::query_as::<_, AccountRow>(
        "SELECT id, name, email, provider_type, auth_type, created_at FROM accounts WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("account {} not found", id)))?;

    Ok(Json(row))
}

pub async fn delete_account(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let rows = sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await?;
    if rows.rows_affected() == 0 {
        return Err(AppError::NotFound(format!("account {} not found", id)));
    }
    Ok(Json(serde_json::json!({ "status": "deleted" })))
}

pub async fn get_account_settings(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let row = sqlx::query(
        "SELECT id, name, email, provider_type, sync_days_limit, signature FROM accounts WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("account {} not found", id)))?;

    use sqlx::Row;
    Ok(Json(serde_json::json!({
        "id": row.get::<String, _>("id"),
        "name": row.get::<String, _>("name"),
        "email": row.get::<String, _>("email"),
        "providerType": row.get::<String, _>("provider_type"),
        "syncDaysLimit": row.get::<Option<i64>, _>("sync_days_limit"),
        "signature": row.get::<Option<String>, _>("signature"),
    })))
}

pub async fn trigger_sync(
    State(state): State<Arc<AppState>>,
    Path(account_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE id = ?")
        .bind(&account_id)
        .fetch_one(&state.pool)
        .await?;
    let exists = count > 0;

    if !exists {
        return Err(AppError::NotFound(format!(
            "account {} not found",
            account_id
        )));
    }

    let orchestrator = SyncOrchestrator::new(state.pool.clone(), state.event_tx.clone());
    orchestrator.sync_account(&account_id).await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
