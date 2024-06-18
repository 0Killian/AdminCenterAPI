use anyhow::Result;
use axum::async_trait;
use sqlx::{MySqlPool, PgPool, SqlitePool};
use tower_sessions::{
    session::{Id, Record},
    session_store, ExpiredDeletion, SessionStore,
};
use tower_sessions_sqlx_store::{MySqlStore, PostgresStore, SqliteStore};

#[derive(Clone, Debug)]
pub enum SqlxPool {
    Sqlite(SqlitePool),
    Postgres(PgPool),
    MySql(MySqlPool),
}

#[derive(Clone, Debug)]
pub enum SqlxSessionStore {
    Sqlite(SqliteStore),
    Postgres(PostgresStore),
    MySql(MySqlStore),
}

impl SqlxSessionStore {
    /// Create a new session store for the provided connection pool
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use sqlx::AnyPool;
    ///
    /// # tokio_test::block_on(async {
    /// let pool = AnyPool::connect("sqlite::memory:").await.unwrap();
    /// let store = AnyStore::new(pool);
    /// # })
    /// ```
    pub fn new(pool: SqlxPool) -> Self {
        match pool {
            SqlxPool::Sqlite(pool) => Self::Sqlite(SqliteStore::new(pool)),
            SqlxPool::Postgres(pool) => Self::Postgres(PostgresStore::new(pool)),
            SqlxPool::MySql(pool) => Self::MySql(MySqlStore::new(pool)),
        }
    }

    /// Migrate the session schema.
    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        match &self {
            SqlxSessionStore::Sqlite(store) => store.migrate().await,
            SqlxSessionStore::Postgres(store) => store.migrate().await,
            SqlxSessionStore::MySql(store) => store.migrate().await,
        }
    }
}

#[async_trait]
impl SessionStore for SqlxSessionStore {
    /// Creates a new session in the store with the provided session record.
    ///
    /// Implementers must decide how to handle potential ID collisions. For
    /// example, they might generate a new unique ID or return `Error::Backend`.
    ///
    /// The record is given as an exclusive reference to allow modifications,
    /// such as assigning a new ID, during the creation process.
    async fn create(&self, session_record: &mut Record) -> session_store::Result<()> {
        match &self {
            SqlxSessionStore::Sqlite(store) => store.create(session_record).await,
            SqlxSessionStore::Postgres(store) => store.create(session_record).await,
            SqlxSessionStore::MySql(store) => store.create(session_record).await,
        }
    }

    /// Saves the provided session record to the store.
    ///
    /// This method is intended for updating the state of an existing session.
    async fn save(&self, session_record: &Record) -> session_store::Result<()> {
        match &self {
            SqlxSessionStore::Sqlite(store) => store.save(session_record).await,
            SqlxSessionStore::Postgres(store) => store.save(session_record).await,
            SqlxSessionStore::MySql(store) => store.save(session_record).await,
        }
    }

    /// Loads an existing session record from the store using the provided ID.
    ///
    /// If a session with the given ID exists, it is returned. If the session
    /// does not exist or has been invalidated (e.g., expired), `None` is
    /// returned.
    async fn load(&self, session_id: &Id) -> session_store::Result<Option<Record>> {
        match &self {
            SqlxSessionStore::Sqlite(store) => store.load(session_id).await,
            SqlxSessionStore::Postgres(store) => store.load(session_id).await,
            SqlxSessionStore::MySql(store) => store.load(session_id).await,
        }
    }

    /// Deletes a session record from the store using the provided ID.
    ///
    /// If the session exists, it is removed from the store.
    async fn delete(&self, session_id: &Id) -> session_store::Result<()> {
        match &self {
            SqlxSessionStore::Sqlite(store) => store.delete(session_id).await,
            SqlxSessionStore::Postgres(store) => store.delete(session_id).await,
            SqlxSessionStore::MySql(store) => store.delete(session_id).await,
        }
    }
}

#[async_trait]
impl ExpiredDeletion for SqlxSessionStore {
    async fn delete_expired(&self) -> session_store::Result<()> {
        match &self {
            SqlxSessionStore::Sqlite(store) => store.delete_expired().await,
            SqlxSessionStore::Postgres(store) => store.delete_expired().await,
            SqlxSessionStore::MySql(store) => store.delete_expired().await,
        }
    }
}
