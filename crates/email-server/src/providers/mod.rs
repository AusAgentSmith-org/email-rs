#![allow(dead_code)]

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Result;

// ── Domain types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub name: String,
    pub full_path: String,
    pub special_use: Option<String>,
    pub unread_count: u32,
    pub total_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub uid: u32,
    pub message_id: Option<String>,
    pub thread_id: Option<String>,
    pub subject: Option<String>,
    pub from_name: Option<String>,
    pub from_email: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub date: Option<DateTime<Utc>>,
    pub is_read: bool,
    pub is_flagged: bool,
    pub is_draft: bool,
    pub has_attachments: bool,
    pub preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBody {
    pub message_id: String,
    pub html_body: Option<String>,
    pub text_body: Option<String>,
    pub raw_headers: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attendee {
    pub email: String,
    pub name: Option<String>,
    pub response_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEvent {
    pub id: String,
    pub calendar_id: String,
    pub provider_event_id: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub location: Option<String>,
    pub is_all_day: bool,
    pub recurrence_rule: Option<String>,
    pub attendees: Vec<Attendee>,
    pub meet_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCalendarEvent {
    pub title: String,
    pub description: Option<String>,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub location: Option<String>,
    pub is_all_day: bool,
    pub attendees: Vec<Attendee>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusySlot {
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

// ── Provider traits ──────────────────────────────────────────────────────────

#[async_trait]
pub trait MailProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    async fn authenticate(&mut self) -> Result<()>;
    async fn list_folders(&self) -> Result<Vec<Folder>>;
    async fn fetch_messages(
        &self,
        folder: &str,
        since: Option<DateTime<Utc>>,
    ) -> Result<Vec<Message>>;
    async fn fetch_message_body(&self, folder: &str, uid: u32) -> Result<MessageBody>;
    /// Batch-fetch bodies for multiple UIDs in a single IMAP session.
    /// Returns (uid, body) pairs; silently skips any UIDs that fail to parse.
    async fn fetch_bodies_batch(
        &self,
        folder: &str,
        uids: &[u32],
    ) -> Result<Vec<(u32, MessageBody)>>;
    /// Set the \Seen flag on the server so the read state is durable across clients.
    async fn mark_seen(&self, folder: &str, uid: u32) -> Result<()>;
    /// Remove the \Seen flag on the server.
    async fn mark_unseen(&self, folder: &str, uid: u32) -> Result<()>;
    /// Move a message to the trash folder (permanent delete on providers without trash).
    async fn delete_message(&self, folder: &str, uid: u32) -> Result<()>;
    /// Move a message to a destination folder by IMAP MOVE (or COPY+STORE \Deleted+EXPUNGE).
    async fn move_message(&self, src_folder: &str, uid: u32, dest_folder: &str) -> Result<()>;
    /// Toggle the \Flagged flag.
    async fn set_flagged(&self, folder: &str, uid: u32, flagged: bool) -> Result<()>;
}

#[async_trait]
pub trait CalendarProvider: Send + Sync {
    fn provider_id(&self) -> &str;
    async fn list_calendars(&self) -> Result<Vec<Calendar>>;
    async fn list_events(
        &self,
        calendar_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>>;
    async fn create_event(
        &self,
        calendar_id: &str,
        event: NewCalendarEvent,
    ) -> Result<CalendarEvent>;
    async fn update_event(&self, calendar_id: &str, event: CalendarEvent) -> Result<CalendarEvent>;
    async fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<()>;
}

/// Optional richer API — Google Calendar, MS Graph etc.
#[async_trait]
pub trait RichCalendarProvider: CalendarProvider {
    async fn get_free_busy(
        &self,
        calendar_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<BusySlot>>;
    async fn create_meeting_link(&self, event: &CalendarEvent) -> Result<Option<String>>;
}

pub mod caldav;
pub mod generic_imap;
pub mod gmail;
pub mod google_calendar;
