use std::sync::Arc;

use axum::{
    extract::{Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::calendar::CalendarService;
use crate::error::Result;
use crate::providers::CalendarEvent;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub account_id: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

pub async fn list_events(
    State(state): State<Arc<AppState>>,
    Query(q): Query<EventsQuery>,
) -> Result<Json<Vec<CalendarEvent>>> {
    let service = CalendarService::new(state.pool.clone());

    let account_id = q.account_id.unwrap_or_default();
    let from = q.from.unwrap_or_else(Utc::now);
    let to = q.to.unwrap_or_else(|| from + chrono::Duration::days(30));

    let events = service.list_events(&account_id, from, to).await?;
    Ok(Json(events))
}
