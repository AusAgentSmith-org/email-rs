# MSI Debug Status — RESOLVED (2026-04-21)

## Root cause

`db/mod.rs` stripped `sqlite:///` and `sqlite://` prefixes before calling `create_dir_all`,
but not the bare `sqlite:` prefix used by the Windows default `DATABASE_URL`:

```
sqlite:C:/Users/sproo/AppData/Local/email-rs/email.db
```

Without stripping `sqlite:`, the path passed to `create_dir_all` was
`sqlite:C:/Users/.../email-rs` — invalid on Windows because `:` is forbidden
in directory names (os error 123). The server thread died immediately.

**Fix:** one extra line in `db/mod.rs`:
```rust
.or_else(|| database_url.strip_prefix("sqlite:").map(String::from))
```

## Secondary issues found and fixed

| Issue | Fix |
|-------|-----|
| No log at startup | `win_log` helper added; errors/panics append to `%TEMP%\email-rs.log` |
| Installer disappeared with no finish dialog | Added `WixUI_Minimal` (WixToolset.UI.wixext/5.0.2) |
| App didn't launch after install | Added `LaunchApp` CustomAction after `InstallFinalize` |

## What was ruled out

- **Service failure** — replaced with system tray app; no service
- **WiX extension mismatch** — `WixToolset.Util.wixext/7.0.0` was incompatible; dropped when `InternetShortcut` removed. `WixToolset.UI.wixext/5.0.2` is now used for the finish dialog
- **Log write permission** — log was in `Program Files` (read-only); moved to `%TEMP%`
- **Tray E_FAIL via SSH** — `Shell_NotifyIcon` returns `0x80004005` in non-interactive (SSH) sessions; works correctly in interactive desktop sessions

## Current installer state

- WiX v5.0.2, no service, no `WixToolset.Util.wixext`
- `WixToolset.UI.wixext/5.0.2` for finish dialog
- `WixUI_Minimal` → shows progress + Finish page
- `LaunchApp` CustomAction → fires tray exe after install
- UpgradeCode `D9E7F2A4-8B1C-4E3D-9F5A-6C2D8E4F1B9A` — must never change

## CI

Pipeline: `.woodpecker/windows-release.yml`
Runner: `loungeroomwinOrg` (Windows, `WoodpeckerAgentUser` scheduled task as `sproo`)
Repo ID: 54
