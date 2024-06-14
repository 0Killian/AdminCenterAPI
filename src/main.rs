use anyhow::{Context, Result};
use axum::{response::IntoResponse, routing::get};
use serde::{Deserialize, Serialize};
use tokio::{signal, task::AbortHandle};
use tower_sessions::{
    cookie::time::Duration, session_store::ExpiredDeletion, Session, SessionManagerLayer,
    SessionStore,
};
use tower_sessions_sqlx_store::{
    sqlx::{Database, MySql, Pool, Postgres, Sqlite},
    MySqlStore, PostgresStore, SqliteStore,
};

mod config;

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

    // Describe the application
    let app = axum::Router::new().route("/", get(index));

    // Hook up the session manager
    let (app, deletion_task) = match &config.database_uri {
        config::DatabaseUri::Sqlite(_) => {
            hook_session_manager::<Sqlite, SqliteStore>(&config, app).await?
        }
        config::DatabaseUri::Postgres(_) => {
            hook_session_manager::<Postgres, PostgresStore>(&config, app).await?
        }
        config::DatabaseUri::Mysql(_) => {
            hook_session_manager::<MySql, MySqlStore>(&config, app).await?
        }
    };

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

// NOTE: tower_sessions_sqlx_store does not provide a way to use the pools and session stores "dynamically"
// (meaning that each type of database has its own pool and session store type). The following code serve
// as a workaround for this.

// TODO: Find a better method for this...
trait SessionStoreFromPool<DB>: SessionStore + ExpiredDeletion + Clone
where
    DB: Database,
{
    fn new(pool: Pool<DB>) -> Self;
    async fn migrate(&self) -> tower_sessions_sqlx_store::sqlx::Result<()>;
}

impl SessionStoreFromPool<Sqlite> for SqliteStore {
    fn new(pool: Pool<Sqlite>) -> Self {
        SqliteStore::new(pool)
    }

    async fn migrate(&self) -> tower_sessions_sqlx_store::sqlx::Result<()> {
        SqliteStore::migrate(self).await
    }
}

impl SessionStoreFromPool<Postgres> for PostgresStore {
    fn new(pool: Pool<Postgres>) -> Self {
        PostgresStore::new(pool)
    }

    async fn migrate(&self) -> tower_sessions_sqlx_store::sqlx::Result<()> {
        PostgresStore::migrate(self).await
    }
}

impl SessionStoreFromPool<MySql> for MySqlStore {
    fn new(pool: Pool<MySql>) -> Self {
        MySqlStore::new(pool)
    }

    async fn migrate(&self) -> tower_sessions_sqlx_store::sqlx::Result<()> {
        MySqlStore::migrate(self).await
    }
}

async fn hook_session_manager<DB, S>(
    config: &config::Config,
    app: axum::Router,
) -> Result<(
    axum::Router,
    tokio::task::JoinHandle<tower_sessions::session_store::Result<()>>,
)>
where
    DB: Database,
    S: SessionStoreFromPool<DB>,
{
    let pool = Pool::<DB>::connect(&config.database_uri.get_connection_string())
        .await
        .context("Failed to connect to sqlite database")?;
    let store = S::new(pool.clone());

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

    Ok((app.layer(session_layer), deletion_task))
}
