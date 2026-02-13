use sqlx::postgres::PgPoolOptions;
use sqlx::ConnectOptions;
use sqlx::PgPool;
use std::time::Duration;

const DEFAULT_MAX_CONNECTIONS: u32 = 10;
const ACQUIRE_TIMEOUT_SECS: u64 = 5;
const IDLE_TIMEOUT_SECS: u64 = 600;
const MAX_LIFETIME_SECS: u64 = 1800;

pub struct DatabaseConfig {
    pub url: String,
}

impl DatabaseConfig {
    pub fn from_env() -> Option<Self> {
        if let Ok(url) = std::env::var("DATABASE_URL") {
            return Some(Self { url });
        }

        let host = std::env::var("POSTGRES_HOST").ok()?;
        let port: u16 = std::env::var("POSTGRES_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(5432);
        let user = std::env::var("POSTGRES_USER").ok()?;
        let password = std::env::var("POSTGRES_PASSWORD").ok()?;
        let db = std::env::var("POSTGRES_DB").ok()?;

        let options = sqlx::postgres::PgConnectOptions::new()
            .host(&host)
            .port(port)
            .username(&user)
            .password(&password)
            .database(&db);

        Some(Self {
            url: options.to_url_lossy().to_string(),
        })
    }

    pub async fn create_pool(&self) -> Result<PgPool, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(DEFAULT_MAX_CONNECTIONS)
            .acquire_timeout(Duration::from_secs(ACQUIRE_TIMEOUT_SECS))
            .idle_timeout(Duration::from_secs(IDLE_TIMEOUT_SECS))
            .max_lifetime(Duration::from_secs(MAX_LIFETIME_SECS))
            .connect(&self.url)
            .await
    }
}
