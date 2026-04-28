# email-rs

A self-hosted email and calendar client. Rust/Axum backend, React/TypeScript frontend, SQLite database. Runs as a single Docker container and is accessible from any browser.

Built as a clean-UI replacement for Thunderbird/Outlook.

---

## Features

**Email**
- Gmail (OAuth2) and Outlook/Exchange (OAuth2 + app-password) accounts
- Generic IMAP/SMTP accounts
- Multi-account support — sync all accounts in one view
- Background IMAP sync with pre-fetching
- Full-text search (SQLite FTS5, BM25 ranked)
- Advanced search with condition builder (AND/OR groups, all message fields)
- Compose with To/Cc/Bcc, reply, reply-all, forward
- Recipient autocomplete from contacts/history
- Labels, snooze, smart folders
- Rules engine — condition-based automatic actions on incoming mail
- Mark read/unread, bulk mark-all-read

**Calendar**
- Google Calendar integration (read + write)
- Week view with event detail
- Calendar events appear in search results

**UI**
- 3-pane layout (sidebar / message list / reading pane)
- Light and dark themes, compact/comfortable density
- Command palette (Ctrl+K) — compose, navigate, search, toggle calendar
- Unread count in browser tab title

**Windows**
- Native Windows app via WebView2 (single-instance, system tray, MSI installer)

**Mobile**
- Companion React Native / Capacitor app (`mobile/`) with inbox, search, compose, and folder navigation

---

## Quick start (Docker)

```bash
cp .env.example .env
# Fill in your OAuth credentials — see .env.example for instructions
docker compose up -d
```

Open **http://localhost:8585**, then go to Settings → Add Account.

The backend hot-reloads via `cargo-watch`; the frontend via Vite HMR. The first start takes ~60s while Rust compiles inside the container.

---

## Configuration

All configuration is via environment variables (`.env` or `docker-compose.yml`).

| Variable | Description | Default |
|---|---|---|
| `GOOGLE_CLIENT_ID` | Google OAuth2 client ID | — |
| `GOOGLE_CLIENT_SECRET` | Google OAuth2 client secret | — |
| `MICROSOFT_CLIENT_ID` | Microsoft OAuth2 client ID | — |
| `MICROSOFT_CLIENT_SECRET` | Microsoft OAuth2 client secret | — |
| `DATABASE_URL` | SQLite connection string | `sqlite://email.db` |
| `HOST` | Bind address | `127.0.0.1` |
| `PORT` | Bind port | `8585` |
| `FRONTEND_DIST` | Path to built frontend assets | `<exe dir>/static` |

### Setting up OAuth credentials

**Google (Gmail + Calendar)**
1. Go to [Google Cloud Console](https://console.cloud.google.com) → APIs & Services → Credentials
2. Create an OAuth 2.0 Client ID (Web application)
3. Add `http://localhost:8585/api/v1/auth/gmail/callback` as an authorised redirect URI
4. Enable the Gmail API and Google Calendar API for your project

**Microsoft (Outlook / Exchange)**
1. Go to [Azure portal](https://portal.azure.com) → App registrations → New registration
2. Add `http://localhost:8585/api/v1/auth/microsoft/callback` as a redirect URI
3. Under API permissions add: `Mail.ReadWrite`, `Mail.Send`, `Calendars.ReadWrite`, `offline_access`

Generic IMAP/SMTP accounts and app-password accounts do not require OAuth setup.

---

## Running without Docker

```bash
# Backend
cargo run -p email-server

# Frontend (separate terminal)
cd frontend && npm ci && npm run dev
```

The backend serves the built frontend from `FRONTEND_DIST`. In dev mode the Caddy proxy in `docker-compose.yml` routes `/api/*` to the backend and everything else to the Vite dev server.

---

## Building a production image

```bash
docker build -t email-rs .
docker run -d \
  -e GOOGLE_CLIENT_ID=... \
  -e GOOGLE_CLIENT_SECRET=... \
  -e MICROSOFT_CLIENT_ID=... \
  -e MICROSOFT_CLIENT_SECRET=... \
  -e DATABASE_URL=sqlite:///data/email.db \
  -v email-data:/data \
  -p 8585:3000 \
  email-rs
```

---

## Windows installer

Pre-built MSI installers are available on the [Releases](../../releases) page.

To build from source you need: Rust (MSVC toolchain), Node.js LTS, .NET SDK 8, and WiX v5.

```powershell
cd frontend && npm ci
npx vite build --outDir ..\staging\static --emptyOutDir
cd ..
cargo build --release --target x86_64-pc-windows-msvc -p email-server
# Then build the MSI with wix (see .woodpecker/windows/installer.wxs)
```

---

## Stack

| Layer | Technology |
|---|---|
| Backend | Rust, Axum, SQLx, SQLite |
| Frontend | React, TypeScript, Vite, Zustand |
| Mobile | React, Capacitor |
| Auth | OAuth2 (Google, Microsoft), IMAP basic/app-password |
| Mail transport | async-imap, lettre |
| Search | SQLite FTS5 with BM25 ranking |
| Dev proxy | Caddy |

---

## License

MIT
