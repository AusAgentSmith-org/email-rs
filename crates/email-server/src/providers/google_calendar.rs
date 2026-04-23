#![allow(dead_code)]

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{
    Attendee, BusySlot, Calendar, CalendarEvent, CalendarProvider, NewCalendarEvent,
    RichCalendarProvider,
};
use crate::error::{AppError, Result};

// ── Google Calendar API response types ───────────────────────────────────────

#[derive(Debug, Deserialize)]
struct GcalListResponse {
    items: Option<Vec<GcalCalendarEntry>>,
}

#[derive(Debug, Deserialize)]
struct GcalCalendarEntry {
    id: String,
    summary: Option<String>,
    #[serde(default)]
    primary: bool,
    #[serde(rename = "backgroundColor")]
    background_color: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GcalEventsResponse {
    items: Option<Vec<GcalEvent>>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GcalEvent {
    id: String,
    summary: Option<String>,
    description: Option<String>,
    location: Option<String>,
    start: Option<GcalDateTime>,
    end: Option<GcalDateTime>,
    recurrence: Option<Vec<String>>,
    attendees: Option<Vec<GcalAttendee>>,
    #[serde(rename = "conferenceData")]
    conference_data: Option<GcalConferenceData>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GcalDateTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GcalAttendee {
    email: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "responseStatus")]
    response_status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GcalConferenceData {
    #[serde(rename = "entryPoints")]
    entry_points: Option<Vec<GcalEntryPoint>>,
}

#[derive(Debug, Deserialize)]
struct GcalEntryPoint {
    #[serde(rename = "entryPointType")]
    entry_point_type: String,
    uri: String,
}

// ── Google Calendar API request types ────────────────────────────────────────

#[derive(Debug, Serialize)]
struct GcalEventRequest {
    summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    location: Option<String>,
    start: GcalDateTimeReq,
    end: GcalDateTimeReq,
    #[serde(skip_serializing_if = "Option::is_none")]
    attendees: Option<Vec<GcalAttendeeReq>>,
}

#[derive(Debug, Serialize)]
struct GcalDateTimeReq {
    #[serde(rename = "dateTime", skip_serializing_if = "Option::is_none")]
    date_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<String>,
}

#[derive(Debug, Serialize)]
struct GcalAttendeeReq {
    email: String,
}

// ── Conversion helpers ────────────────────────────────────────────────────────

fn parse_gcal_datetime(dt: &GcalDateTime) -> (DateTime<Utc>, bool) {
    if let Some(s) = &dt.date_time {
        let parsed = DateTime::parse_from_rfc3339(s)
            .map(|d| d.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        (parsed, false)
    } else if let Some(s) = &dt.date {
        let naive = NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Local::now().date_naive());
        let dt = naive.and_hms_opt(0, 0, 0).expect("valid hms");
        let utc = DateTime::from_naive_utc_and_offset(dt, Utc);
        (utc, true)
    } else {
        (Utc::now(), false)
    }
}

fn gcal_event_to_domain(g: GcalEvent, calendar_id: &str) -> Option<CalendarEvent> {
    // Skip cancelled events
    if g.status.as_deref() == Some("cancelled") {
        return None;
    }

    let start = g.start.as_ref()?;
    let end = g.end.as_ref()?;
    let (start_at, is_all_day) = parse_gcal_datetime(start);
    let (end_at, _) = parse_gcal_datetime(end);

    let meet_link = g
        .conference_data
        .as_ref()
        .and_then(|cd| cd.entry_points.as_ref())
        .and_then(|eps| eps.iter().find(|ep| ep.entry_point_type == "video"))
        .map(|ep| ep.uri.clone());

    let attendees = g
        .attendees
        .unwrap_or_default()
        .into_iter()
        .map(|a| Attendee {
            email: a.email,
            name: a.display_name,
            response_status: a.response_status,
        })
        .collect();

    let recurrence_rule = g
        .recurrence
        .and_then(|rs| rs.into_iter().find(|r| r.starts_with("RRULE:")));

    Some(CalendarEvent {
        id: Uuid::new_v4().to_string(),
        calendar_id: calendar_id.to_string(),
        provider_event_id: Some(g.id),
        title: g.summary.unwrap_or_else(|| "(no title)".to_string()),
        description: g.description,
        start_at,
        end_at,
        location: g.location,
        is_all_day,
        recurrence_rule,
        attendees,
        meet_link,
    })
}

fn new_event_to_gcal_request(event: &NewCalendarEvent) -> GcalEventRequest {
    let (start, end) = if event.is_all_day {
        (
            GcalDateTimeReq {
                date_time: None,
                date: Some(event.start_at.format("%Y-%m-%d").to_string()),
            },
            GcalDateTimeReq {
                date_time: None,
                date: Some(event.end_at.format("%Y-%m-%d").to_string()),
            },
        )
    } else {
        (
            GcalDateTimeReq {
                date_time: Some(event.start_at.to_rfc3339()),
                date: None,
            },
            GcalDateTimeReq {
                date_time: Some(event.end_at.to_rfc3339()),
                date: None,
            },
        )
    };

    GcalEventRequest {
        summary: event.title.clone(),
        description: event.description.clone(),
        location: event.location.clone(),
        start,
        end,
        attendees: if event.attendees.is_empty() {
            None
        } else {
            Some(
                event
                    .attendees
                    .iter()
                    .map(|a| GcalAttendeeReq {
                        email: a.email.clone(),
                    })
                    .collect(),
            )
        },
    }
}

// ── GoogleCalendarProvider ────────────────────────────────────────────────────

pub struct GoogleCalendarProvider {
    pub account_id: String,
    pub email: String,
    access_token: String,
    client: Client,
}

impl GoogleCalendarProvider {
    pub fn new(account_id: String, email: String, access_token: String) -> Self {
        Self {
            account_id,
            email,
            access_token,
            client: Client::new(),
        }
    }

    fn auth(&self) -> String {
        format!("Bearer {}", self.access_token)
    }

    async fn check(&self, resp: reqwest::Response) -> Result<reqwest::Response> {
        if resp.status().is_success() {
            return Ok(resp);
        }
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        if status == 401 {
            return Err(AppError::Auth(format!(
                "Google Calendar: unauthorized — {}",
                body
            )));
        }
        Err(AppError::Provider(format!(
            "Google Calendar API error {}: {}",
            status, body
        )))
    }
}

#[async_trait]
impl CalendarProvider for GoogleCalendarProvider {
    fn provider_id(&self) -> &str {
        &self.account_id
    }

    async fn list_calendars(&self) -> Result<Vec<Calendar>> {
        let resp = self
            .client
            .get("https://www.googleapis.com/calendar/v3/users/me/calendarList")
            .header("Authorization", self.auth())
            .send()
            .await
            .map_err(|e| AppError::Provider(e.to_string()))?;

        let resp = self.check(resp).await?;
        let list: GcalListResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Provider(e.to_string()))?;

        Ok(list
            .items
            .unwrap_or_default()
            .into_iter()
            .map(|c| Calendar {
                id: c.id,
                name: c.summary.unwrap_or_else(|| "Calendar".to_string()),
                color: c.background_color,
                is_primary: c.primary,
            })
            .collect())
    }

    async fn list_events(
        &self,
        calendar_id: &str,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<CalendarEvent>> {
        let cal_enc = urlencoding::encode(calendar_id).into_owned();
        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            cal_enc
        );

        let mut all: Vec<CalendarEvent> = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut req = self
                .client
                .get(&url)
                .header("Authorization", self.auth())
                .query(&[
                    ("timeMin", from.to_rfc3339()),
                    ("timeMax", to.to_rfc3339()),
                    ("singleEvents", "true".to_string()),
                    ("orderBy", "startTime".to_string()),
                    ("maxResults", "250".to_string()),
                ]);

            if let Some(tok) = &page_token {
                req = req.query(&[("pageToken", tok.as_str())]);
            }

            let resp = req
                .send()
                .await
                .map_err(|e| AppError::Provider(e.to_string()))?;
            let resp = self.check(resp).await?;
            let page: GcalEventsResponse = resp
                .json()
                .await
                .map_err(|e| AppError::Provider(e.to_string()))?;

            for g in page.items.unwrap_or_default() {
                if let Some(event) = gcal_event_to_domain(g, calendar_id) {
                    all.push(event);
                }
            }

            match page.next_page_token {
                Some(tok) => page_token = Some(tok),
                None => break,
            }
        }

        Ok(all)
    }

    async fn create_event(
        &self,
        calendar_id: &str,
        event: NewCalendarEvent,
    ) -> Result<CalendarEvent> {
        let cal_enc = urlencoding::encode(calendar_id).into_owned();
        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            cal_enc
        );

        let body = new_event_to_gcal_request(&event);

        let resp = self
            .client
            .post(&url)
            .header("Authorization", self.auth())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider(e.to_string()))?;

        let resp = self.check(resp).await?;
        let g: GcalEvent = resp
            .json()
            .await
            .map_err(|e| AppError::Provider(e.to_string()))?;

        gcal_event_to_domain(g, calendar_id)
            .ok_or_else(|| AppError::Provider("invalid event in create response".to_string()))
    }

    async fn update_event(&self, calendar_id: &str, event: CalendarEvent) -> Result<CalendarEvent> {
        let provider_id = event.provider_event_id.as_deref().ok_or_else(|| {
            AppError::Provider("event has no provider_event_id, cannot update".to_string())
        })?;

        let cal_enc = urlencoding::encode(calendar_id).into_owned();
        let evt_enc = urlencoding::encode(provider_id).into_owned();
        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
            cal_enc, evt_enc
        );

        let new_ev = NewCalendarEvent {
            title: event.title.clone(),
            description: event.description.clone(),
            start_at: event.start_at,
            end_at: event.end_at,
            location: event.location.clone(),
            is_all_day: event.is_all_day,
            attendees: event.attendees.clone(),
        };
        let body = new_event_to_gcal_request(&new_ev);

        let resp = self
            .client
            .put(&url)
            .header("Authorization", self.auth())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Provider(e.to_string()))?;

        let resp = self.check(resp).await?;
        let g: GcalEvent = resp
            .json()
            .await
            .map_err(|e| AppError::Provider(e.to_string()))?;

        let mut updated = gcal_event_to_domain(g, calendar_id)
            .ok_or_else(|| AppError::Provider("invalid event in update response".to_string()))?;
        updated.id = event.id; // preserve local ID
        Ok(updated)
    }

    async fn delete_event(&self, calendar_id: &str, provider_event_id: &str) -> Result<()> {
        let cal_enc = urlencoding::encode(calendar_id).into_owned();
        let evt_enc = urlencoding::encode(provider_event_id).into_owned();
        let url = format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events/{}",
            cal_enc, evt_enc
        );

        let resp = self
            .client
            .delete(&url)
            .header("Authorization", self.auth())
            .send()
            .await
            .map_err(|e| AppError::Provider(e.to_string()))?;

        if resp.status().as_u16() == 204 || resp.status().is_success() {
            return Ok(());
        }
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        Err(AppError::Provider(format!(
            "Google Calendar delete error {}: {}",
            status, body
        )))
    }
}

#[async_trait]
impl RichCalendarProvider for GoogleCalendarProvider {
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
