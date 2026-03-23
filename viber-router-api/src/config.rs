use anyhow::{Context, Result};

pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub redis_url: String,
    pub redis_max_connections: usize,
    pub rust_log: String,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .context("DATABASE_URL is required")?;
        let redis_url = std::env::var("REDIS_URL")
            .context("REDIS_URL is required")?;

        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".into())
            .parse::<u16>()
            .context("PORT must be a valid u16")?;
        let database_max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "50".into())
            .parse::<u32>()
            .context("DATABASE_MAX_CONNECTIONS must be a valid u32")?;
        let redis_max_connections = std::env::var("REDIS_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "30".into())
            .parse::<usize>()
            .context("REDIS_MAX_CONNECTIONS must be a valid usize")?;
        let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());

        Ok(Self {
            host,
            port,
            database_url,
            database_max_connections,
            redis_url,
            redis_max_connections,
            rust_log,
        })
    }
}
