use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;

use accounting_service::config::DatabaseConfig;
use accounting_service::handlers::{
    create_account, delete_account, get_account, list_accounts, update_account,
    DynAccountRepository,
};
use accounting_service::repository::{InMemoryAccountRepository, PostgresAccountRepository};

#[tokio::main]
async fn main() {
    common::init_tracing();

    let _ = dotenvy::dotenv();

    let repo: DynAccountRepository = match DatabaseConfig::from_env() {
        Some(config) => {
            tracing::info!("Connecting to PostgreSQL...");
            let pool = config
                .create_pool()
                .await
                .expect("Failed to connect to PostgreSQL");

            sqlx::migrate!("./migrations")
                .run(&pool)
                .await
                .expect("Failed to run database migrations");

            tracing::info!("PostgreSQL connected and migrations applied");
            Arc::new(PostgresAccountRepository::new(pool))
        }
        None => {
            tracing::warn!("DATABASE_URL not set, using in-memory repository");
            Arc::new(InMemoryAccountRepository::new())
        }
    };

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/accounts", post(create_account).get(list_accounts))
        .route(
            "/api/accounts/:id",
            get(get_account).put(update_account).delete(delete_account),
        )
        .with_state(repo);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8082));
    tracing::info!("accounting-service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "service": "accounting-service",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn health() -> &'static str {
    "OK"
}
