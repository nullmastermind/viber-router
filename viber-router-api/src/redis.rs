use anyhow::{Context, Result};
use deadpool_redis::{Config as RedisConfig, Pool, Runtime};

use crate::config::Config;

pub fn create_pool(config: &Config) -> Result<Pool> {
    let mut cfg = RedisConfig::from_url(&config.redis_url);
    cfg.pool = Some(deadpool_redis::PoolConfig {
        max_size: config.redis_max_connections,
        ..Default::default()
    });
    cfg.create_pool(Some(Runtime::Tokio1))
        .context("Failed to create Redis pool")
}
