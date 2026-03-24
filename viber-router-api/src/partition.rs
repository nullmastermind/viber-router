use chrono::{Datelike, NaiveDate, Utc};
use sqlx::PgPool;

/// Ensure partitions exist for the given months.
pub async fn ensure_partitions(pool: &PgPool) {
    let now = Utc::now().naive_utc().date();
    let current = (now.year(), now.month());
    let next = next_month(now.year(), now.month());

    create_partition(pool, current.0, current.1).await;
    create_partition(pool, next.0, next.1).await;
}

async fn create_partition(pool: &PgPool, year: i32, month: u32) {
    let (next_year, next_month) = next_month(year, month);
    let name = format!("proxy_logs_{year}_{month:02}");
    let from = format!("{year}-{month:02}-01");
    let to = format!("{next_year}-{next_month:02}-01");

    let sql = format!(
        "CREATE TABLE IF NOT EXISTS \"{name}\" PARTITION OF proxy_logs \
         FOR VALUES FROM ('{from}') TO ('{to}')"
    );

    if let Err(e) = sqlx::query(&sql).execute(pool).await {
        tracing::warn!("Failed to create partition {name}: {e}");
    } else {
        tracing::debug!("Ensured partition {name}");
    }
}

/// Drop partitions older than retention_days.
pub async fn drop_expired_partitions(pool: &PgPool, retention_days: u32) {
    let cutoff = Utc::now().naive_utc().date() - chrono::Duration::days(i64::from(retention_days));

    // List partition tables matching our naming convention
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT tablename FROM pg_tables \
         WHERE schemaname = 'public' AND tablename LIKE 'proxy_logs_%' \
         AND tablename != 'proxy_logs' \
         ORDER BY tablename"
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for table in rows {
        if let Some(end_date) = parse_partition_end_date(&table)
            && end_date <= cutoff
        {
            let sql = format!("DROP TABLE IF EXISTS \"{table}\"");
            if let Err(e) = sqlx::query(&sql).execute(pool).await {
                tracing::warn!("Failed to drop expired partition {table}: {e}");
            } else {
                tracing::info!("Dropped expired partition {table}");
            }
        }
    }
}

fn next_month(year: i32, month: u32) -> (i32, u32) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

/// Parse "proxy_logs_YYYY_MM" → end date of that month (first day of next month).
fn parse_partition_end_date(name: &str) -> Option<NaiveDate> {
    let suffix = name.strip_prefix("proxy_logs_")?;
    let mut parts = suffix.splitn(2, '_');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let (ny, nm) = next_month(year, month);
    NaiveDate::from_ymd_opt(ny, nm, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_month() {
        assert_eq!(next_month(2026, 3), (2026, 4));
        assert_eq!(next_month(2026, 12), (2027, 1));
    }

    #[test]
    fn test_parse_partition_end_date() {
        assert_eq!(
            parse_partition_end_date("proxy_logs_2026_03"),
            NaiveDate::from_ymd_opt(2026, 4, 1)
        );
        assert_eq!(
            parse_partition_end_date("proxy_logs_2026_12"),
            NaiveDate::from_ymd_opt(2027, 1, 1)
        );
        assert_eq!(parse_partition_end_date("other_table"), None);
    }
}
