use anyhow::{Context, Result};
use axum::{response::IntoResponse, routing::get};
use serde::{Deserialize, Serialize};
use session_store::{SqlxPool, SqlxSessionStore};
use sqlx::{MySqlPool, PgPool, SqlitePool};
use tokio::{signal, task::AbortHandle};
use tower_sessions::{
    cookie::time::Duration, session_store::ExpiredDeletion, Session, SessionManagerLayer,
};

mod config;
mod session_store;

// States
#[derive(Serialize, Deserialize, Default)]
struct Counter(usize);

// Handlers
async fn index(session: Session) -> impl IntoResponse {
    let counter: Counter = session.get("counter").await.unwrap().unwrap_or_default();
    session
        .insert("counter", Counter(counter.0 + 1))
        .await
        .unwrap();
    format!("Hello {}!", counter.0)
}

// Configuration for the session layer
const SESSION_LAYER_SECURE: bool = false;
const SESSION_STORE_EXPIRATION: Duration = Duration::minutes(20);

#[tokio::main]
async fn main() -> Result<()> {
    // Load config based on the environment
    dotenv::dotenv().ok();
    let config = config::Config::from_env()?;

    // Connect to the database
    let pool = match config.database_uri {
        config::DatabaseUri::Sqlite(_) => SqlxPool::Sqlite({
            let pool = SqlitePool::connect(&config.database_uri.get_connection_string()).await?;
            sqlx::migrate!("migrations/sqlite").run(&pool).await?;
            pool
        }),
        config::DatabaseUri::Postgres(_) => SqlxPool::Postgres({
            let pool = PgPool::connect(&config.database_uri.get_connection_string()).await?;
            sqlx::migrate!("migrations/postgres").run(&pool).await?;
            pool
        }),
        config::DatabaseUri::Mysql(_) => SqlxPool::MySql({
            let pool = MySqlPool::connect(&config.database_uri.get_connection_string()).await?;
            sqlx::migrate!("migrations/mysql").run(&pool).await?;
            pool
        }),
    };

    // Create the session store
    let store = SqlxSessionStore::new(pool.clone());

    store
        .migrate()
        .await
        .with_context(|| "Failed to migrate session store")?;

    let deletion_task = tokio::task::spawn(
        store
            .clone()
            .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
    );

    let session_layer = SessionManagerLayer::new(store)
        .with_secure(SESSION_LAYER_SECURE)
        .with_expiry(tower_sessions::Expiry::OnInactivity(
            SESSION_STORE_EXPIRATION,
        ));

    // Describe the application
    let app = axum::Router::new()
        .route("/", get(index))
        .layer(session_layer);

    // Start the server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await
        .unwrap();

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
        .await
        .unwrap();

    // Wait for the deletion task to finish
    deletion_task.await??;

    Ok(())
}

// Aborts the deletion task when the server is shut down
async fn shutdown_signal(abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to register signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { abort_handle.abort() },
        _ = terminate => { abort_handle.abort() },
    }
}
