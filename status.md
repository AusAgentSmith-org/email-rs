# email-rs — Project Status

**Started:** 2026-04-19  
**Status:** Active development — core features functional, sync working, search live

---

## Goal

A self-hosted email + calendar client to replace Thunderbird/Outlook. The web GUI is the primary interface — not a TUI or native app. Accessible from any browser, self-hosted.

Primary pain point with existing clients: Thunderbird's GUI is poor. This project prioritises a clean, high-fidelity UI as a first-class concern.

---

## Design

Design delivered as a high-fidelity HTML prototype: `design_handoff_email_client/Email Client.html`.

**Layout:** Classic 3-pane (sidebar 200px | message list 280px | reading pane flex:1)  
**Themes:** Light + dark, full token system  
**Density modes:** Compact (36px rows) / Cozy (52px, default) / Comfy (68px)  
**Typography:** Inter (UI) + JetBrains Mono (timestamps, email addresses, filenames)  
**Design tokens:** CSS custom properties in `frontend/src/styles/tokens.css` — exact values from prototype

---

## Architecture Decisions

### Stack
- **Backend:** Rust, Axum, SQLite (sqlx), async-imap, lettre
- **Frontend:** React 18 + TypeScript + Vite — the main player
- **State:** Zustand (persists theme + density to localStorage)
- **Styling:** CSS modules + CSS custom properties — no Tailwind (design is too custom)

### Provider trait system
All mail and calendar integrations are behind provider traits:

```
MailProvider        — core IMAP operations (all providers)
CalendarProvider    — core CalDAV/calendar CRUD (all providers)
RichCalendarProvider: CalendarProvider — enhanced APIs (Google, MS Graph)
```

### Calendar approach
CalDAV core + optional provider-specific API enhancement. Not locked to Google Calendar API.

### Auth
- OAuth2 for Google, Microsoft
- Basic auth / App passwords for everything else (Fastmail, self-hosted)
- Token refresh handled in the provider impl layer

### Offline cache
Full offline cache — all message headers/metadata always synced to SQLite. Message bodies fetched lazily on first open then cached.

### Search architecture
- **FTS5 virtual table** (`messages_fts`) with `unicode61` tokenizer — word-boundary tokenisation (no substring false-positives like "syntax" matching "tax")
- **BM25 ranking** with column weights: subject=10, from_name=5, from_email=5, preview=2
- **Sync triggers** keep FTS index in sync automatically on INSERT/UPDATE/DELETE
- **Autocomplete** endpoint (`/search/suggest`) uses FTS5 prefix queries (`"term"*`) — returns 8 results
- LIKE fallback if FTS5 is unavailable

### Rules engine (planned)
Conditions reuse the same `ConditionGroup`/`Condition` model as advanced search — a rule is just a saved condition group + action list.

### Deployment
Self-hosted. Backend serves the React SPA from `frontend/dist/` as static files. Single binary + SQLite.

Dev stack: `docker compose up --build` — Caddy proxy on :8585, backend (`cargo watch`), frontend (Vite HMR).

---

## Module Status

### Backend

| Module | Status | Notes |
|--------|--------|-------|
| `main.rs` | Done | Axum router, pool init, static file serving, sync spawn |
| `config.rs` | Done | Env-driven config |
| `error.rs` | Done | `AppError` thiserror + `IntoResponse` |
| `db/` | Done | Runtime migrations 001–004 + FTS5 migration (005, inline Rust) |
| `providers/mod.rs` | Done | All three traits + domain types |
| `providers/gmail.rs` | Working | OAuth2 token refresh, IMAP XOAUTH2, SMTP |
| `providers/caldav.rs` | Skeleton | HTTP basic auth stub |
| `auth/` | Working | OAuth2 URL + token exchange, callback handler |
| `imap/sync.rs` | Working | Poll loop wired to real IMAP, 4-connection parallel sync |
| `smtp/mod.rs` | Skeleton | lettre multipart send stub |
| `calendar/mod.rs` | **Done** | `CalendarService`: list/get/upsert events, email-event link CRUD |
| `calendar/sync.rs` | **Done** | Google Calendar sync triggered after IMAP sync (best-effort) |
| `providers/google_calendar.rs` | **Done** | Full CalendarProvider + CRUD, pagination, Meet link extraction |
| `sync/mod.rs` | Working | `SyncOrchestrator` with SSE broadcast |
| `api/accounts` | Working | CRUD + trigger sync |
| `api/folders` | Working | List, patch, mark-read |
| `api/messages` | Working | List, get (lazy body fetch), patch, delete, archive, bulk |
| `api/search` | **Done** | FTS5 + BM25 ranking + LIKE fallback; `/suggest` autocomplete endpoint |
| `api/smart_folders` | Done | all / unread / flagged |
| `api/compose` | **Done** | POST /messages sends via lettre (Gmail XOAUTH2 + basic auth) |
| `api/calendar` | **Done** | List/get events, link management (add/remove/list email↔event) |
| `api/events` | Done | SSE broadcast for sync progress |
| `api/webhooks` | Scaffold | Table + routes, delivery not implemented |

### Frontend

| Module | Status | Notes |
|--------|--------|-------|
| `tokens.css` | Done | Full light + dark token system |
| `store/index.ts` | Done | Zustand; persists theme + density; conditionGroup for advanced search |
| `types/index.ts` | Done | Message, Folder, Account, CalendarEvent, Condition types, Suggestion |
| `App.tsx` | Done | 3-pane grid, resizable panels, data-theme + data-density |
| `Sidebar.tsx` | Done | Compose, search + autocomplete dropdown, folder nav, account strip |
| `MessageList.tsx` | Done | Header, scrollable list, thread grouping, bulk actions, keyboard nav |
| `MessageRow.tsx` | Done | Avatar, sender, timestamp, unread dot |
| `ReadingPane.tsx` | Done | Header, lazy body fetch, quick reply |
| `ConditionBuilder` | **Done** | Shared condition builder for search + rules (field/operator/value rows) |
| `AdvancedSearchModal` | **Done** | Visual advanced search panel (Gmail-style) |
| `SettingsModal` | Done | Account settings, folder exclusions |
| `ComposeModal` | Done | Compose + send (text body, reply/forward, signature) |
| `Calendar/CalendarView` | **Done** | 7-day week grid, event chips, all-day row, prev/next/today nav |
| `Calendar/EventDetail` | **Done** | Event detail panel, attendees, Meet link, linked emails |
| `useApi.ts` | Done | Abort-controller fetch hook |
| `utils/search.ts` | Done | `conditionGroupToSearchUrl` serialiser |

---

## Database Schema

| Table | Purpose |
|-------|---------|
| `accounts` | Email account credentials + provider config |
| `folders` | IMAP folders per account, sync state |
| `messages` | Message metadata + headers (always synced) |
| `message_bodies` | HTML/text body (fetched lazily, cached) |
| `attachments` | Attachment blobs |
| `calendar_events` | Unified calendar event cache (synced from Google Calendar) |
| `message_calendar_links` | Many-to-many email ↔ calendar event links |
| `webhooks` | Event webhook config |
| `messages_fts` | FTS5 virtual table — subject, from_name, from_email, preview |

---

## What's Working Now

- Gmail OAuth2 full flow (authorize → callback → token storage → refresh)
- IMAP sync: 4 connections in parallel, 197 folders, incremental sync every 5 minutes
- SSE sync progress broadcast → frontend progress strip
- Message list, lazy body fetch, mark read/flagged, archive, delete, bulk actions
- Thread grouping in folder view
- **FTS5 search** — word-boundary tokenization, BM25 relevance (subject > from > preview > body)
- **Autocomplete dropdown** — 150ms debounce, prefix FTS5, keyboard nav, click-to-navigate
- **Advanced search modal** — visual condition builder (from/to/subject/date/attachment filters), All/Any match
- Smart folders: All Inboxes, Unread, Flagged
- **Compose send** — Gmail XOAUTH2 + basic auth SMTP, reply/forward with quoted text + signature
- **Google Calendar** — syncs after IMAP sync, 7-day week view, event detail with Meet link + attendees
- **Email↔event linking** — link/unlink emails to calendar events, click linked email to navigate to it
- **Search across mail + calendar** — FTS5 + BM25 for messages, LIKE for events; unified autocomplete dropdown
- Dark/light theme, 3 density modes, resizable panels

---

## Windows Distribution

MSI installer built and shipped via Woodpecker CI (`loungeroomwinOrg` agent, Windows native).

| Thing | Status |
|-------|--------|
| Rust binary (MSVC target) | Done — release build, `windows_subsystem = "windows"` |
| Native WebView window (tao 0.30 + wry 0.47) | Done — opens on launch, 1280×800, min 800×600 |
| System tray (tray-icon 0.19) | Done — icon, Open/Quit menu, left-click restores window |
| MSI installer (WiX v5.0.2) | Done — WixUI_Minimal finish dialog, auto-launches app after install |
| Error log | `%TEMP%\email-rs.log` — panic hook + explicit error paths write here |
| CI pipeline | `.woodpecker/windows-release.yml` — triggers on tag or manual |

**Behaviour:**
- Launch → Axum server starts in background thread; app polls until server is ready (up to 10s), then opens a native WebView2 window showing the React UI.
- Close button → hides to tray (does not quit). Tray left-click or "Open email-rs" menu → shows and focuses the window.
- Tray "Quit" → exits the process and server.
- Browser access (`http://localhost:<port>`) still works as a secondary access method.
- Requires Microsoft Edge WebView2 Runtime (bundled with Windows 10 21H1+ and Windows 11).
- App does **not** add itself to startup — launch from Start Menu or add `email-server.exe` to `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` manually if wanted.

**Bugs fixed (2026-04-21):**
- `db/mod.rs` wasn't stripping the bare `sqlite:` prefix before `create_dir_all` — the `C:` drive letter made the path invalid on Windows (os error 123). Fixed by adding `.or_else(|| url.strip_prefix("sqlite:"))` to the prefix chain.

---

## What's Not Built Yet

- Rules / filters engine (conditions model done, execution in progress)
- Snooze (in progress)
- Labels / tags (in progress)
- CalDAV sync (providers/caldav.rs is a skeleton)
- Linux/macOS deployment (Komodo stack)
- Mobile-responsive layout

---

## Immediate Next Steps

1. Rules engine — merge in-progress backend + frontend
2. Snooze — merge in-progress backend + frontend
3. Labels — merge in-progress backend + frontend
4. Deployment — Woodpecker pipeline + Komodo stack in ops repo
