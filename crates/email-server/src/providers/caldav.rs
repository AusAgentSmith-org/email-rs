#![allow(dead_code)]

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use super::{
    BusySlot, Calendar, CalendarEvent, CalendarProvider, NewCalendarEvent, RichCalendarProvider,
};
use crate::error::Result;

pub struct CalDavProvider {
    pub account_id: String,
    pub base_url: String,
    pub username: String,
    pub password: String,
    client: reqwest::Client,
}

impl CalDavProvider {
    pub fn new(account_id: String, base_url: String, username: String, password: String) -> Self {
        Self {
            account_id,
            base_url,
            username,
            password,
            client: reqwest::Client::new(),
        }
    }

    fn auth_header(&self) -> String {
        use base64::Engine as _;
        let credentials = format!("{}:{}", self.username, self.password);
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
        format!("Basic {}", encoded)
    }
}

#[async_trait]
impl CalendarProvider for CalDavProvider {
    fn provider_id(&self) -> &str {
        "caldav"
    }

    async fn list_calendars(&self) -> Result<Vec<Calendar>> {
        // Full implementation: PROPFIND request to CalDAV server.
        let _ = self.auth_header();
        Ok(vec![Calendar {
            id: "default".to_string(),
            name: "Calendar".to_string(),
            color: None,
            is_primary: true,
        }])
    }

    async fn list_events(
        &self,
        _calendar_id: &str,
        _from: DateTime<Utc>,
        _to: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>> {
        Ok(vec![])
    }

    async fn create_event(
        &self,
        calendar_id: &str,
        event: NewCalendarEvent,
    ) -> Result<CalendarEvent> {
        Ok(CalendarEvent {
            id: uuid::Uuid::new_v4().to_string(),
            calendar_id: calendar_id.to_string(),
            provider_event_id: None,
            title: event.title,
            description: event.description,
            start_at: event.start_at,
            end_at: event.end_at,
            location: event.location,
            is_all_day: event.is_all_day,
            recurrence_rule: None,
            attendees: event.attendees,
            meet_link: None,
        })
    }

    async fn update_event(
        &self,
        _calendar_id: &str,
        event: CalendarEvent,
    ) -> Result<CalendarEvent> {
        Ok(event)
    }

    async fn delete_event(&self, _calendar_id: &str, _event_id: &str) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl RichCalendarProvider for CalDavProvider {
    async fn get_free_busy(
        &self,
        _calendar_id: &str,
        _from: DateTime<Utc>,
        _to: DateTime<Utc>,
    ) -> Result<Vec<BusySlot>> {
        Ok(vec![])
    }

    async fn create_meeting_link(&self, _event: &CalendarEvent) -> Result<Option<String>> {
        Ok(None)
    }
}
