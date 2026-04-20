use lettre::{
    message::{header::ContentType, MultiPart, SinglePart},
    transport::smtp::authentication::{Credentials, Mechanism},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    /// For XOAUTH2: the OAuth2 access token. For Basic: the password.
    pub password: String,
    pub use_tls: bool,
    /// Use XOAUTH2 mechanism (Gmail OAuth2) instead of PLAIN/LOGIN.
    pub xoauth2: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundMessage {
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub in_reply_to: Option<String>,
}

pub async fn send_message(config: &SmtpConfig, msg: OutboundMessage) -> Result<()> {
    let from = msg
        .from
        .parse()
        .map_err(|e: lettre::address::AddressError| AppError::Smtp(e.to_string()))?;

    let mut builder = Message::builder().from(from).subject(&msg.subject);

    for to in &msg.to {
        let addr = to
            .parse()
            .map_err(|e: lettre::address::AddressError| AppError::Smtp(e.to_string()))?;
        builder = builder.to(addr);
    }

    if let Some(reply_to) = &msg.in_reply_to {
        builder = builder.in_reply_to(reply_to.clone());
    }

    let email = match (&msg.text_body, &msg.html_body) {
        (Some(text), Some(html)) => builder
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text.clone()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html.clone()),
                    ),
            )
            .map_err(|e| AppError::Smtp(e.to_string()))?,
        (Some(text), None) => builder
            .body(text.clone())
            .map_err(|e| AppError::Smtp(e.to_string()))?,
        (None, Some(html)) => builder
            .header(ContentType::TEXT_HTML)
            .body(html.clone())
            .map_err(|e| AppError::Smtp(e.to_string()))?,
        (None, None) => return Err(AppError::Smtp("message has no body".to_string())),
    };

    let creds = Credentials::new(config.username.clone(), config.password.clone());
    let mechanisms = if config.xoauth2 {
        vec![Mechanism::Xoauth2]
    } else {
        vec![Mechanism::Plain, Mechanism::Login]
    };

    let transport = if config.use_tls {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)
            .map_err(|e| AppError::Smtp(e.to_string()))?
            .credentials(creds)
            .authentication(mechanisms)
            .build()
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)
            .map_err(|e| AppError::Smtp(e.to_string()))?
            .port(config.port)
            .credentials(creds)
            .authentication(mechanisms)
            .build()
    };

    transport
        .send(email)
        .await
        .map_err(|e| AppError::Smtp(e.to_string()))?;

    Ok(())
}
