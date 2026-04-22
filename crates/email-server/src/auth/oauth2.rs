#![allow(dead_code)]

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
    pub token_type: String,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    /// Unix timestamp (seconds) when the access token expires.
    pub expires_at: Option<i64>,
}

impl StoredToken {
    pub fn from_token_response(resp: TokenResponse) -> Self {
        let expires_at = resp
            .expires_in
            .map(|secs| Utc::now().timestamp() + secs as i64);
        Self {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            expires_at,
        }
    }

    /// Returns true if the access token is expired or will expire within 5 minutes.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(exp) => Utc::now().timestamp() >= exp - 300,
            None => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub public_client: bool,
}

impl OAuthConfig {
    pub fn gmail(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            redirect_uri,
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
                "https://mail.google.com/".to_string(),
                "https://www.googleapis.com/auth/calendar".to_string(),
            ],
            public_client: false,
        }
    }

    pub fn microsoft(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client_id,
            client_secret,
            auth_url: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize".to_string(),
            token_url: "https://login.microsoftonline.com/common/oauth2/v2.0/token".to_string(),
            redirect_uri,
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
                "offline_access".to_string(),
                "https://outlook.office.com/IMAP.AccessAsUser.All".to_string(),
                "https://outlook.office.com/SMTP.Send".to_string(),
                "https://graph.microsoft.com/User.Read".to_string(),
            ],
            public_client: true,
        }
    }

    pub fn authorization_url(&self, state: &str) -> String {
        let scopes = self.scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&access_type=offline&prompt=consent",
            self.auth_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scopes),
            urlencoding::encode(state),
        )
    }

    /// Authorization URL for Microsoft — omits Google-specific `access_type` param.
    pub fn authorization_url_microsoft(&self, state: &str) -> String {
        let scopes = self.scopes.join(" ");
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&prompt=consent",
            self.auth_url,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&scopes),
            urlencoding::encode(state),
        )
    }

    pub async fn exchange_code(&self, code: &str) -> Result<TokenResponse> {
        let client = reqwest::Client::new();
        let scope = self.scopes.join(" ");
        let mut params = vec![
            ("code", code),
            ("client_id", self.client_id.as_str()),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
            ("scope", scope.as_str()),
        ];
        if !self.public_client {
            params.push(("client_secret", self.client_secret.as_str()));
        }

        let body = client
            .post(&self.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::Auth(e.to_string()))?
            .text()
            .await
            .map_err(|e| AppError::Auth(e.to_string()))?;

        serde_json::from_str::<TokenResponse>(&body)
            .map_err(|e| AppError::Auth(format!("token parse error: {} — body: {}", e, body)))
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        let client = reqwest::Client::new();
        let mut params = vec![
            ("refresh_token", refresh_token),
            ("client_id", self.client_id.as_str()),
            ("grant_type", "refresh_token"),
        ];
        if !self.public_client {
            params.push(("client_secret", self.client_secret.as_str()));
        }

        let body = client
            .post(&self.token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::Auth(e.to_string()))?
            .text()
            .await
            .map_err(|e| AppError::Auth(e.to_string()))?;

        serde_json::from_str::<TokenResponse>(&body)
            .map_err(|e| AppError::Auth(format!("token parse error: {} — body: {}", e, body)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn token_expiring_in(offset_secs: i64) -> StoredToken {
        StoredToken {
            access_token: "tok".into(),
            refresh_token: Some("ref".into()),
            expires_at: Some(Utc::now().timestamp() + offset_secs),
        }
    }

    #[test]
    fn not_expired_when_no_expiry() {
        let t = StoredToken {
            access_token: "tok".into(),
            refresh_token: None,
            expires_at: None,
        };
        assert!(!t.is_expired());
    }

    #[test]
    fn not_expired_when_far_future() {
        assert!(!token_expiring_in(3600).is_expired());
    }

    #[test]
    fn expired_when_past() {
        assert!(token_expiring_in(-1).is_expired());
    }

    #[test]
    fn expired_within_5_min_grace_window() {
        // 4 minutes remaining is inside the 300s grace window → treat as expired
        assert!(token_expiring_in(240).is_expired());
    }

    #[test]
    fn not_expired_just_outside_grace_window() {
        // 6 minutes remaining is outside the 300s window → still valid
        assert!(!token_expiring_in(360).is_expired());
    }

    #[test]
    fn from_token_response_computes_expires_at() {
        let resp = TokenResponse {
            access_token: "a".into(),
            refresh_token: Some("r".into()),
            expires_in: Some(3600),
            token_type: "Bearer".into(),
            scope: None,
        };
        let before = Utc::now().timestamp();
        let stored = StoredToken::from_token_response(resp);
        let after = Utc::now().timestamp();
        let exp = stored.expires_at.unwrap();
        assert!(exp >= before + 3600 && exp <= after + 3600);
    }

    #[test]
    fn from_token_response_no_expiry_when_expires_in_missing() {
        let resp = TokenResponse {
            access_token: "a".into(),
            refresh_token: None,
            expires_in: None,
            token_type: "Bearer".into(),
            scope: None,
        };
        assert!(StoredToken::from_token_response(resp).expires_at.is_none());
    }

    #[test]
    fn gmail_auth_url_includes_required_params() {
        let cfg = OAuthConfig::gmail("my-cid".into(), "sec".into(), "http://localhost/cb".into());
        let url = cfg.authorization_url("state123");
        assert!(url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"));
        assert!(url.contains("client_id=my-cid"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("state=state123"));
        assert!(url.contains("prompt=consent"));
    }

    #[test]
    fn microsoft_auth_url_omits_access_type() {
        let cfg =
            OAuthConfig::microsoft("ms-cid".into(), "sec".into(), "http://localhost/cb".into());
        let url = cfg.authorization_url_microsoft("st456");
        assert!(url.starts_with("https://login.microsoftonline.com/common/oauth2/v2.0/authorize?"));
        assert!(!url.contains("access_type"));
        assert!(url.contains("client_id=ms-cid"));
        assert!(url.contains("state=st456"));
    }

    #[test]
    fn state_with_special_chars_is_url_encoded() {
        let cfg = OAuthConfig::gmail("id".into(), "sec".into(), "http://localhost/cb".into());
        let url = cfg.authorization_url("has spaces&symbols");
        // Raw unencoded string must not appear
        assert!(!url.contains("has spaces&symbols"));
    }
}
