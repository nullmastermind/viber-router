use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Model {
    pub id: Uuid,
    pub name: String,
    pub input_1m_usd: Option<f64>,
    pub output_1m_usd: Option<f64>,
    pub cache_write_1m_usd: Option<f64>,
    pub cache_read_1m_usd: Option<f64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateModel {
    pub name: String,
    pub input_1m_usd: Option<f64>,
    pub output_1m_usd: Option<f64>,
    pub cache_write_1m_usd: Option<f64>,
    pub cache_read_1m_usd: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModel {
    pub name: Option<String>,
    pub input_1m_usd: Option<Option<f64>>,
    pub output_1m_usd: Option<Option<f64>>,
    pub cache_write_1m_usd: Option<Option<f64>>,
    pub cache_read_1m_usd: Option<Option<f64>>,
}
