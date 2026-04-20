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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = Config::from_env();
    info!("starting email-server on {}", cfg.bind_addr());

    let (pool, has_fts) = db::create_pool(&cfg.database_url).await?;

    let app_state = Arc::new(AppState::new(pool.clone(), has_fts));

    // Spawn background sync, sharing the same event_tx as AppState
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
