use std::sync::Arc;

use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::state::AppState;

pub mod accounts;
pub mod auth;
pub mod calendar;
pub mod compose;
pub mod events;
pub mod folders;
pub mod messages;
pub mod search;
pub mod smart_folders;
pub mod webhooks;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/accounts",
            get(accounts::list_accounts).post(accounts::create_account),
        )
        .route(
            "/accounts/{id}",
            patch(accounts::update_account).delete(accounts::delete_account),
        )
        .route(
            "/accounts/{id}/settings",
            get(accounts::get_account_settings),
        )
        .route("/accounts/{id}/folders", get(folders::list_folders))
        .route("/folders/{id}", patch(folders::patch_folder))
        .route("/folders/{id}/mark-read", post(folders::mark_folder_read))
        .route("/folders/{id}/messages", get(messages::list_messages))
        .route("/messages/bulk", post(messages::bulk_messages))
        .route(
            "/messages/{id}",
            get(messages::get_message)
                .patch(messages::patch_message)
                .delete(messages::delete_message),
        )
        .route("/messages/{id}/archive", post(messages::archive_message))
        .route("/messages", post(compose::send_message))
        .route("/calendar/events", get(calendar::list_events))
        .route("/calendar/events/{id}", get(calendar::get_event))
        .route(
            "/calendar/events/{id}/links",
            get(calendar::list_event_links).post(calendar::add_event_link),
        )
        .route(
            "/calendar/events/{id}/links/{message_id}",
            axum::routing::delete(calendar::remove_event_link),
        )
        .route("/sync/{account_id}", post(accounts::trigger_sync))
        .route("/auth/gmail/authorize", get(auth::gmail_authorize))
        .route("/auth/gmail/callback", get(auth::gmail_callback))
        .route("/auth/microsoft/authorize", get(auth::microsoft_authorize))
        .route("/auth/microsoft/callback", get(auth::microsoft_callback))
        .route("/events", get(events::sse_events))
        .route("/search", get(search::search_messages))
        .route("/search/suggest", get(search::suggest_messages))
        .route(
            "/smart-folders/{kind}/messages",
            get(smart_folders::list_smart_messages),
        )
        .route(
            "/webhooks",
            get(webhooks::list_webhooks).post(webhooks::create_webhook),
        )
        .route(
            "/webhooks/{id}",
            patch(webhooks::update_webhook).delete(webhooks::delete_webhook),
        )
        .with_state(state)
}
