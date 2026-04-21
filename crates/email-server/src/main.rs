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

        // Catch panics and write them to the log before the process dies
        let panic_log = log_path.clone();
        std::panic::set_hook(Box::new(move |info| {
            let msg = format!("panic: {info}");
            let _ = std::fs::write(&panic_log, &msg);
        }));

        let port = cfg.port;
        let server_log = log_path.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
            if let Err(e) = rt.block_on(run_server(cfg)) {
                let _ = std::fs::write(&server_log, format!("server error: {e:#}"));
            }
        });
        if let Err(e) = run_tray(port) {
            let _ = std::fs::write(&log_path, format!("tray error: {e:#}"));
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
fn run_tray(port: u16) -> anyhow::Result<()> {
    use std::time::{Duration, Instant};
    use tray_icon::{
        menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
        MouseButton, TrayIconBuilder, TrayIconEvent,
    };
    use winit::{
        application::ApplicationHandler,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
        window::WindowId,
    };

    let open_item = MenuItem::new("Open email-rs", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let open_id = open_item.id().clone();
    let quit_id = quit_item.id().clone();

    let menu = Menu::new();
    menu.append_items(&[&open_item, &PredefinedMenuItem::separator(), &quit_item])?;

    // Simple 32×32 blue icon
    let icon_rgba: Vec<u8> = (0..32u32 * 32)
        .flat_map(|_| [0x26u8, 0x8B, 0xD2, 0xFF])
        .collect();

    let url = format!("http://localhost:{}", port);

    struct TrayApp {
        tray: Option<tray_icon::TrayIcon>,
        menu: Option<tray_icon::menu::Menu>,
        icon_rgba: Vec<u8>,
        open_id: tray_icon::menu::MenuId,
        quit_id: tray_icon::menu::MenuId,
        url: String,
    }

    impl ApplicationHandler for TrayApp {
        fn resumed(&mut self, _el: &ActiveEventLoop) {
            if self.tray.is_none() {
                let icon =
                    tray_icon::Icon::from_rgba(self.icon_rgba.clone(), 32, 32).expect("valid icon");
                if let Some(menu) = self.menu.take() {
                    self.tray = TrayIconBuilder::new()
                        .with_menu(Box::new(menu))
                        .with_tooltip("email-rs")
                        .with_icon(icon)
                        .build()
                        .ok();
                }
            }
        }

        fn window_event(&mut self, _el: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {}

        fn about_to_wait(&mut self, el: &ActiveEventLoop) {
            el.set_control_flow(ControlFlow::WaitUntil(
                Instant::now() + Duration::from_millis(50),
            ));

            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id == self.open_id {
                    let _ = webbrowser::open(&self.url);
                } else if event.id == self.quit_id {
                    el.exit();
                }
            }

            if let Ok(event) = TrayIconEvent::receiver().try_recv() {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    ..
                } = event
                {
                    let _ = webbrowser::open(&self.url);
                }
            }
        }
    }

    let event_loop = EventLoop::new()?;
    let mut app = TrayApp {
        tray: None,
        menu: Some(menu),
        icon_rgba,
        open_id,
        quit_id,
        url,
    };
    event_loop.run_app(&mut app)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn exe_log_path() -> std::path::PathBuf {
    // Program Files is read-only for normal users; TEMP is always writable
    let base = std::env::var("TEMP")
        .or_else(|_| std::env::var("TMP"))
        .unwrap_or_else(|_| "C:\\Windows\\Temp".into());
    std::path::PathBuf::from(base).join("email-rs.log")
}
