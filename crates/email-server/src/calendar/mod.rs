use chrono::{DateTime, Utc};
use sqlx::SqlitePool;

use crate::error::Result;
use crate::providers::CalendarEvent;

#[derive(Debug, sqlx::FromRow)]
struct CalendarEventRow {
    id: String,
    calendar_id: String,
    provider_event_id: Option<String>,
    title: String,
    description: Option<String>,
    start_at: String,
    end_at: String,
    location: Option<String>,
    is_all_day: bool,
    recurrence_rule: Option<String>,
    attendees_json: Option<String>,
    meet_link: Option<String>,
}

/// Dispatch calendar operations to the appropriate provider implementation
/// and keep the local SQLite cache in sync.
pub struct CalendarService {
    pool: SqlitePool,
}

impl CalendarService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_events(
        &self,
        account_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>> {
        let from_str = from.to_rfc3339();
        let to_str = to.to_rfc3339();

        let rows = sqlx::query_as::<_, CalendarEventRow>(
            r#"SELECT id, calendar_id, provider_event_id, title, description,
                      start_at, end_at, location, is_all_day, recurrence_rule,
                      attendees_json, meet_link
               FROM calendar_events
               WHERE account_id = ?
                 AND start_at >= ?
                 AND end_at   <= ?
               ORDER BY start_at"#,
        )
        .bind(account_id)
        .bind(&from_str)
        .bind(&to_str)
        .fetch_all(&self.pool)
        .await?;

        let events = rows
            .into_iter()
            .map(|r| CalendarEvent {
                id: r.id,
                calendar_id: r.calendar_id,
                provider_event_id: r.provider_event_id,
                title: r.title,
                description: r.description,
                start_at: r
                    .start_at
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
                end_at: r
                    .end_at
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
                location: r.location,
                is_all_day: r.is_all_day,
                recurrence_rule: r.recurrence_rule,
                attendees: r
                    .attendees_json
                    .as_deref()
                    .and_then(|j| serde_json::from_str(j).ok())
                    .unwrap_or_default(),
                meet_link: r.meet_link,
            })
            .collect();

        Ok(events)
    }
}
