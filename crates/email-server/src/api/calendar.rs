use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::calendar::{CalendarService, EventLink};
use crate::error::{AppError, Result};
use crate::providers::CalendarEvent;
use crate::state::AppState;

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub account_id: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

// ── List events ───────────────────────────────────────────────────────────────

pub async fn list_events(
    State(state): State<Arc<AppState>>,
    Query(q): Query<EventsQuery>,
) -> Result<Json<Vec<CalendarEvent>>> {
    let service = CalendarService::new(state.pool.clone());

    let from = q.from.unwrap_or_else(Utc::now);
    let to = q.to.unwrap_or_else(|| from + chrono::Duration::days(30));

    let events = service
        .list_events(q.account_id.as_deref(), from, to)
        .await?;
    Ok(Json(events))
}

// ── Get single event ──────────────────────────────────────────────────────────

pub async fn get_event(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<CalendarEvent>> {
    let service = CalendarService::new(state.pool.clone());
    let event = service
        .get_event(&id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("event {} not found", id)))?;
    Ok(Json(event))
}

// ── Link management ───────────────────────────────────────────────────────────

pub async fn list_event_links(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<EventLink>>> {
    let service = CalendarService::new(state.pool.clone());
    let links = service.list_event_links(&id).await?;
    Ok(Json(links))
}

#[derive(Debug, Deserialize)]
pub struct AddLinkBody {
    #[serde(rename = "messageId")]
    pub message_id: String,
}

#[derive(Debug, Serialize)]
pub struct AddLinkResponse {
    pub id: String,
}

pub async fn add_event_link(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<AddLinkBody>,
) -> Result<Json<AddLinkResponse>> {
    let service = CalendarService::new(state.pool.clone());
    let link_id = service.add_event_link(&id, &body.message_id).await?;
    Ok(Json(AddLinkResponse { id: link_id }))
}

pub async fn remove_event_link(
    State(state): State<Arc<AppState>>,
    Path((id, message_id)): Path<(String, String)>,
) -> Result<StatusCode> {
    let service = CalendarService::new(state.pool.clone());
    service.remove_event_link(&id, &message_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
