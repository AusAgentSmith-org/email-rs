use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub frontend_dist: String,
}

impl Config {
    pub fn from_env() -> Self {
        let frontend_dist = env::var("FRONTEND_DIST")
            .unwrap_or_else(|_| exe_dir().join("static").to_string_lossy().into_owned());

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            #[cfg(target_os = "windows")]
            {
                let base = env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
                // SQLite path with forward slashes
                format!("sqlite:{}/email-rs/email.db", base.replace('\\', "/"))
            }
            #[cfg(not(target_os = "windows"))]
            {
                "sqlite://email.db".to_string()
            }
        });

        Self {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8585),
            database_url,
            frontend_dist,
        }
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn exe_dir() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."))
}
