#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod api;
mod auth;
mod calendar;
mod config;
mod db;
mod error;
mod imap;
mod providers;
mod smtp;
mod state;
mod sync;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::get_service, Router};
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::Config;
use crate::state::AppState;
use crate::sync::SyncOrchestrator;

fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let cfg = Config::from_env();

    #[cfg(target_os = "windows")]
    {
        let log_path = exe_log_path();

        let panic_log = log_path.clone();
        std::panic::set_hook(Box::new(move |info| {
            win_log(&panic_log, &format!("panic: {info}"));
        }));

        let port = cfg.port;
        let server_log = log_path.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            if let Err(e) = rt.block_on(run_server(cfg)) {
                win_log(&server_log, &format!("server error: {e:#}"));
            }
        });

        if let Err(e) = run_desktop(port, &log_path) {
            win_log(&log_path, &format!("desktop error: {e:#}"));
        }
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        tracing_subscriber::registry()
            .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
            .with(tracing_subscriber::fmt::layer())
            .init();
        tokio::runtime::Runtime::new()?.block_on(run_server(cfg))?;
    }

    Ok(())
}

async fn run_server(cfg: Config) -> anyhow::Result<()> {
    info!("starting email-server on {}", cfg.bind_addr());

    let (pool, has_fts) = db::create_pool(&cfg.database_url).await?;
    let app_state = Arc::new(AppState::new(pool.clone(), has_fts));
    SyncOrchestrator::spawn_background(pool, app_state.event_tx.clone());

    let api_routes = api::router(app_state);
    let static_files = ServeDir::new(&cfg.frontend_dist).append_index_html_on_directories(true);

    let app = Router::new()
        .nest("/api/v1", api_routes)
        .fallback_service(get_service(static_files))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = cfg.bind_addr().parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn run_desktop(port: u16, log: &std::path::Path) -> anyhow::Result<()> {
    use std::time::{Duration, Instant};
    use tao::{
        dpi::LogicalSize,
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    };
    use tray_icon::{
        menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
        MouseButton, TrayIconBuilder, TrayIconEvent,
    };
    use wry::WebViewBuilder;

    let url = format!("http://localhost:{port}");

    let open_item = MenuItem::new("Open email-rs", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let open_id = open_item.id().clone();
    let quit_id = quit_item.id().clone();

    let menu = Menu::new();
    menu.append_items(&[&open_item, &PredefinedMenuItem::separator(), &quit_item])?;

    let icon_rgba: Vec<u8> = (0..32u32 * 32)
        .flat_map(|_| [0x26u8, 0x8Bu8, 0xD2u8, 0xFFu8])
        .collect();

    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("email-rs")
        .with_icon(tray_icon::Icon::from_rgba(icon_rgba, 32, 32)?)
        .build()?;

    // Wait for the Axum server to bind before opening the window
    wait_for_server_ready(port, log);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("email-rs")
        .with_inner_size(LogicalSize::new(1280u32, 800u32))
        .with_min_inner_size(LogicalSize::new(800u32, 600u32))
        .build(&event_loop)?;

    let _webview = WebViewBuilder::new(&window).with_url(&url).build()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));

        if let Ok(ev) = MenuEvent::receiver().try_recv() {
            if ev.id == open_id {
                window.set_visible(true);
                window.focus_window();
            } else if ev.id == quit_id {
                *control_flow = ControlFlow::Exit;
            }
        }

        if let Ok(ev) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                ..
            } = ev
            {
                window.set_visible(true);
                window.focus_window();
            }
        }

        // Close button minimises to tray rather than quitting
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            window.set_visible(false);
        }
    });
}

#[cfg(target_os = "windows")]
fn wait_for_server_ready(port: u16, log: &std::path::Path) {
    use std::time::Duration;
    let addr = format!("127.0.0.1:{port}");
    for _ in 0..100 {
        if std::net::TcpStream::connect(&addr).is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    win_log(
        log,
        "server did not become ready within 10s — opening window anyway",
    );
}

#[cfg(target_os = "windows")]
fn exe_log_path() -> std::path::PathBuf {
    // Program Files is read-only for normal users; TEMP is always writable
    let base = std::env::var("TEMP")
        .or_else(|_| std::env::var("TMP"))
        .unwrap_or_else(|_| "C:\\Windows\\Temp".into());
    std::path::PathBuf::from(base).join("email-rs.log")
}

#[cfg(target_os = "windows")]
fn win_log(path: &std::path::Path, msg: &str) {
    use std::io::Write;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("[{ts}] {msg}\n");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = f.write_all(line.as_bytes());
    }
}
