#![allow(dead_code)]

pub mod oauth2;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthConfig {
    OAuth2 {
        client_id: String,
        client_secret: String,
        access_token: Option<String>,
        refresh_token: Option<String>,
        expires_at: Option<i64>,
    },
    Basic {
        username: String,
        password: String,
    },
    AppPassword {
        email: String,
        app_password: String,
    },
}
