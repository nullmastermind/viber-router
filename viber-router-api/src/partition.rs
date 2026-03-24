use chrono::{Datelike, NaiveDate, Utc};
use sqlx::PgPool;

/// Ensure partitions exist for the given table for current and next month.
pub async fn ensure_partitions(pool: &PgPool, table: &str) {
    let now = Utc::now().naive_utc().date();
    let current = (now.year(), now.month());
    let next = next_month(now.year(), now.month());

    create_partition(pool, table, current.0, current.1).await;
    create_partition(pool, table, next.0, next.1).await;
}

async fn create_partition(pool: &PgPool, table: &str, year: i32, month: u32) {
    let (next_year, next_month) = next_month(year, month);
    let name = format!("{table}_{year}_{month:02}");
    let from = format!("{year}-{month:02}-01");
    let to = format!("{next_year}-{next_month:02}-01");

    let sql = format!(
        "CREATE TABLE IF NOT EXISTS \"{name}\" PARTITION OF {table} \
         FOR VALUES FROM ('{from}') TO ('{to}')"
    );

    if let Err(e) = sqlx::query(&sql).execute(pool).await {
        tracing::warn!("Failed to create partition {name}: {e}");
    } else {
        tracing::debug!("Ensured partition {name}");
    }
}

/// Drop partitions older than retention_days for the given table.
pub async fn drop_expired_partitions(pool: &PgPool, table: &str, retention_days: u32) {
    let cutoff = Utc::now().naive_utc().date() - chrono::Duration::days(i64::from(retention_days));
    let pattern = format!("{table}_%");

    let rows = sqlx::query_scalar::<_, String>(
        "SELECT tablename FROM pg_tables \
         WHERE schemaname = 'public' AND tablename LIKE $1 \
         AND tablename != $2 \
         ORDER BY tablename"
    )
    .bind(&pattern)
    .bind(table)
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    for partition in rows {
        if let Some(end_date) = parse_partition_end_date(&partition, table)
            && end_date <= cutoff
        {
            let sql = format!("DROP TABLE IF EXISTS \"{partition}\"");
            if let Err(e) = sqlx::query(&sql).execute(pool).await {
                tracing::warn!("Failed to drop expired partition {partition}: {e}");
            } else {
                tracing::info!("Dropped expired partition {partition}");
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

/// Parse "{table}_YYYY_MM" → end date of that month (first day of next month).
fn parse_partition_end_date(name: &str, table: &str) -> Option<NaiveDate> {
    let prefix = format!("{table}_");
    let suffix = name.strip_prefix(&prefix)?;
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
    fn test_parse_partition_end_date_proxy_logs() {
        assert_eq!(
            parse_partition_end_date("proxy_logs_2026_03", "proxy_logs"),
            NaiveDate::from_ymd_opt(2026, 4, 1)
        );
        assert_eq!(
            parse_partition_end_date("proxy_logs_2026_12", "proxy_logs"),
            NaiveDate::from_ymd_opt(2027, 1, 1)
        );
        assert_eq!(parse_partition_end_date("other_table", "proxy_logs"), None);
    }

    #[test]
    fn test_parse_partition_end_date_ttft_logs() {
        assert_eq!(
            parse_partition_end_date("ttft_logs_2026_03", "ttft_logs"),
            NaiveDate::from_ymd_opt(2026, 4, 1)
        );
        assert_eq!(
            parse_partition_end_date("ttft_logs_2026_12", "ttft_logs"),
            NaiveDate::from_ymd_opt(2027, 1, 1)
        );
        assert_eq!(parse_partition_end_date("proxy_logs_2026_03", "ttft_logs"), None);
    }
}
