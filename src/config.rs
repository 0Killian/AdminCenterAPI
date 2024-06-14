//! Global configuration for the backend
//! The backend is configured through the environment variables. The recommended way of setting these
//! variables is through the `.env` file. See `.env.sample` for an example.

use anyhow::Result;

/// An URI to a database in the format `scheme://user[:password]@host[:port]/database`
pub struct CommonSqlUri {
    /// The username to use
    user: String,
    /// The password to use, if any
    password: Option<String>,
    /// The host to connect to
    host: String,
    /// The port to connect to
    port: String,
    /// The database to connect to
    database: String,
}

impl CommonSqlUri {
    /// Parse a CommonSqlUri from the given connection string (without the scheme)
    pub fn parse(uri: &str) -> Result<CommonSqlUri> {
        let mut parts = uri.split("@");
        let authentication = parts
            .next()
            .ok_or(anyhow::anyhow!("Malformed database uri"))?;
        let location = parts
            .next()
            .ok_or(anyhow::anyhow!("Malformed database uri"))?;

        let mut parts = authentication.split(':');
        let user = parts
            .next()
            .ok_or(anyhow::anyhow!("Malformed database uri"))?
            .to_string();
        let password = parts.next().map(|p| p.to_string());

        let mut parts = location.split('/');
        let host = parts
            .next()
            .ok_or(anyhow::anyhow!("Malformed database uri"))?
            .to_string();
        let database = parts
            .next()
            .ok_or(anyhow::anyhow!("Malformed database uri"))?
            .to_string();

        let mut parts = host.split(':');
        let host = parts
            .next()
            .ok_or(anyhow::anyhow!("Malformed database uri"))?
            .to_string();
        let port = parts
            .next()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "5432".to_string());

        Ok(CommonSqlUri {
            user,
            password,
            host,
            port,
            database,
        })
    }

    /// Get the connection string for the database (without the scheme)
    pub fn get_connection_string(&self) -> String {
        format!(
            "{}{}@{}:{}/{}",
            self.user,
            self.password
                .as_ref()
                .map(|x| format!(":{}", x))
                .unwrap_or_default(),
            self.host,
            self.port,
            self.database
        )
    }
}

/// The URI to the database, depending on the database type
pub enum DatabaseUri {
    /// The URI to a sqlite database (parsed from sqlite://path)
    Sqlite(String),
    /// The URI to a postgres database (parsed from postgresql://user[:password]@host[:port]/database)
    Postgres(CommonSqlUri),
    /// The URI to a mysql database (parsed from mysql://user[:password]@host[:port]/database)
    Mysql(CommonSqlUri),
}

impl DatabaseUri {
    /// Parse a DatabaseUri from the given connection string
    pub fn parse(uri: String) -> Result<DatabaseUri> {
        let mut parts = uri.split("://");

        match parts.next().unwrap() {
            "sqlite" => {
                let path = parts
                    .next()
                    .ok_or(anyhow::anyhow!("Missing path while parsing database uri"))?;
                Ok(DatabaseUri::Sqlite(path.to_string()))
            }
            "postgresql" => Ok(DatabaseUri::Postgres(CommonSqlUri::parse(
                parts
                    .next()
                    .ok_or(anyhow::anyhow!("Malformed postgresql uri"))?,
            )?)),
            "mysql" => Ok(DatabaseUri::Mysql(CommonSqlUri::parse(
                parts.next().ok_or(anyhow::anyhow!("Malformed mysql uri"))?,
            )?)),
            _ => Err(anyhow::anyhow!("Unknown database type")),
        }
    }

    /// Get the connection string for the database
    pub fn get_connection_string(&self) -> String {
        match self {
            DatabaseUri::Sqlite(path) => {
                format!("sqlite://{}", path)
            }
            DatabaseUri::Postgres(uri) => {
                format!("postgresql://{}", uri.get_connection_string())
            }
            DatabaseUri::Mysql(uri) => {
                format!("mysql://{}", uri.get_connection_string())
            }
        }
    }
}

/// The configuration used by the backend
pub struct Config {
    /// The URI to the database
    pub database_uri: DatabaseUri,
    /// The host to bind to
    pub host: String,
    /// The port to bind to
    pub port: u16,
}

impl Config {
    /// Load the configuration from the environment
    pub fn from_env() -> Result<Config> {
        let raw_database_uri =
            std::env::var("DATABASE_URI").map_err(|_| anyhow::anyhow!("Missing DATABASE_URI"))?;

        let host = std::env::var("HOST").unwrap_or("0.0.0.0".to_string());

        let port = std::env::var("PORT")
            .unwrap_or("3000".to_string())
            .parse()?;

        let database_uri = DatabaseUri::parse(raw_database_uri)?;

        Ok(Config {
            database_uri,
            host,
            port,
        })
    }
}
