# email-rs — Claude Context

Self-hosted email + calendar client (Rust/Axum backend + React/TypeScript frontend). Replaces Thunderbird/Outlook. See `status.md` for full feature state.

## Running Locally

```bash
docker compose up --build -d
# App: http://localhost:8585
# Backend hot-reloads via cargo-watch; frontend via Vite HMR
```

The backend compiles inside the container — first start takes ~60s. Subsequent file saves recompile in ~10s.

## Key Conventions

### Backend
- All handlers return `Result<Json<T>>` using the shared `AppError` type in `error.rs`
- DB migrations run at startup from `db/mod.rs`. Migrations 001–004 are SQL files split by `;`. Migration 005 (FTS5) is written inline in Rust — triggers contain embedded semicolons that break the naive splitter.
- `AppState` is `Arc<AppState>` — add new shared state there, not as separate Axum state
- `has_fts: bool` on AppState — always check this before using FTS5 queries; provide a LIKE fallback

### Frontend
- CSS modules + CSS custom properties from `tokens.css` — no Tailwind, no component library
- Global state in Zustand (`store/index.ts`). Persist only what should survive a page refresh (currently theme + density)
- `useApi(url, { immediate })` for data fetching — re-fetches automatically when `url` changes
- Components live in `src/components/<Name>/<Name>.tsx` + `<Name>.module.css`

### Shared condition model
`ConditionField`, `ConditionOperator`, `Condition`, `ConditionGroup` in `types/index.ts` are the shared model for both advanced search and the (upcoming) rules engine. Do not duplicate this logic — the `ConditionBuilder` component is reusable.

## Search Architecture

- FTS5 virtual table `messages_fts` indexes: subject, from_name, from_email, preview
- Triggers (`messages_fts_ai`, `_au`, `_ad`) keep it in sync automatically
- Main search (`/api/v1/search`): FTS5 MATCH + BM25 weights (subject=10, from=5/5, preview=2), limit 50
- Suggest (`/api/v1/search/suggest`): prefix query (`"term"*`), limit 8, returns `SuggestRow`
- `utils/search.ts`: `conditionGroupToSearchUrl()` — serialises a ConditionGroup to search URL params; includes `q=` for current backend compat plus structured params for future use

## Planned Features (not started)

### Rules engine
- Backend: `rules`, `rule_conditions`, `rule_actions`, `rule_run_log` tables (new migration)
- Conditions reuse `ConditionGroup` — evaluate against incoming messages on sync
- Dry-run mode: return what would match without executing actions
- Shadow mode: log matches but don't act (for new rules on live traffic)
- Frontend: reuse `ConditionBuilder` for the condition editor

### Command palette (Ctrl+K)
- Action registry in Zustand or a plain static map
- Fuzzy match over actions + folder names + contacts
- Dispatch into existing store actions
- Wire "run rule now" and "advanced search" as palette actions

## File Map (key paths)

```
crates/email-server/src/
  main.rs                  — entry point
  state.rs                 — AppState (pool, has_fts, event_tx)
  db/mod.rs                — pool creation + all migrations incl. FTS5
  api/mod.rs               — Axum router
  api/search.rs            — search_messages + suggest_messages + FTS helpers
  api/messages.rs          — MessageRow struct (reused by search)
  providers/gmail.rs       — Gmail IMAP + OAuth2 + SMTP
  imap/sync.rs             — sync orchestration

frontend/src/
  store/index.ts           — Zustand store (conditionGroup, navigateToMessage, etc.)
  types/index.ts           — all TS types incl. Condition* and Suggestion
  utils/search.ts          — conditionGroupToSearchUrl
  components/
    Sidebar/               — search input + autocomplete dropdown + advanced search trigger
    ConditionBuilder/      — shared condition row builder (search + rules)
    AdvancedSearch/        — AdvancedSearchModal wrapping ConditionBuilder
    MessageList/           — message list, thread grouping, bulk actions
```
