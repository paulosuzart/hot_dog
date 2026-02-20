#[cfg(feature = "server")]
use dioxus::server::ServerFnError;

#[cfg(feature = "server")]
use base64::{engine::general_purpose::URL_SAFE, Engine as _};

#[cfg(feature = "server")]
use crate::{
    backend::kids::{Cursor, Granularity, HistoryRow},
    models::{KidHistory, KidHistoryResponse},
};

#[cfg(feature = "server")]
pub struct KidHistoryQuery {
    kid_id: u32,
    cursor: Option<Cursor>,
    page_size: u8,
    granularity: Granularity,
}

#[cfg(feature = "server")]
impl KidHistoryQuery {
    pub fn new(kid_id: u32, cursor: Option<Cursor>, page_size: u8, granularity: Granularity) -> Self {
        Self {
            kid_id,
            cursor: cursor,
            page_size,
            granularity,
        }
    }

    fn cursor_clause(&self) -> String {
        if let Some(cursor) = &self.cursor {
            format!(
                " AND {} > '{}' ",
                self.granularity.grain_format(),
                cursor.grain_value
            )
        } else {
            "".to_string()
        }
    }

    pub async fn execute(&self) -> Result<KidHistoryResponse, ServerFnError> {
        use crate::backend::turso::get_db;

        let conn = get_db().await;

        let mut trx = conn
            .transaction_with_behavior(libsql::TransactionBehavior::Deferred)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let cursor_clause = self.cursor_clause();
        let grain_format = self.granularity.grain_format();

        let total = self
            .fetch_total(&mut trx, &cursor_clause, &grain_format)
            .await?;

        if total == 0 {
            return Ok(KidHistoryResponse {
                history: vec![],
                cursor: None,
                granularity: grain_format.to_string(),
                total_count: 0,
            });
        }

        let mut periods = self.fetch_periods(&mut trx, &cursor_clause).await?;

        let next_cursor = self.extract_next_cursor(&mut periods);

        Ok(KidHistoryResponse {
            history: periods,
            cursor: next_cursor,
            granularity: grain_format.to_string(),
            total_count: total,
        })
    }

    fn extract_next_cursor(&self, periods: &mut Vec<KidHistory>) -> Option<String> {
        if periods.len() > self.page_size as usize {
            let last_item = periods.pop().unwrap();
            let cursor_str = format!("{}|{}", last_item.period, self.granularity);
            Some(URL_SAFE.encode(cursor_str.as_bytes()))
        } else {
            None
        }
    }

    async fn fetch_periods(
        &self,
        trx: &mut libsql::Transaction,
        cursor_clause: &str,
    ) -> Result<Vec<KidHistory>, ServerFnError> {
        use crate::models::KidHistory;

        let query = format!(
            "
            WITH all_stats as ( SELECT
                strftime('{}', notes.created_at) AS period,
                SUM(quantity) AS total,
            COUNT(
                CASE
                WHEN quantity = -1 THEN 1
                END
            ) neg_count,
            COUNT(
                CASE
                WHEN quantity = 1 THEN 1
                END
            ) post_count,
            kids.id AS kid_id,
            kids.name AS name
        FROM kids
        LEFT JOIN notes ON notes.kid_id = kids.id
        WHERE
            kid_id = :kid_id
            {}
        GROUP BY
            period
        ORDER BY
            period DESC)

        select *, (neg_count + post_count) as result from all_stats lIMIT :limit
",
            self.granularity.grain_format(),
            cursor_clause
        );

        tracing::debug!("SQL get_paged_history: {}", query);

        let stm = trx
            .prepare(&query)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let mut rows = stm
            // TODO add grain value as named param
            .query(libsql::named_params! { ":limit": self.page_size + 1, ":kid_id": self.kid_id })
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let mut kid_history: Vec<KidHistory> = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
        {
            use libsql::de;

            let kid =
                de::from_row::<HistoryRow>(&row).map_err(|e| ServerFnError::new(e.to_string()))?;
            kid_history.push(kid.into());
        }

        Ok(kid_history)
    }

    async fn fetch_total(
        &self,
        trx: &mut libsql::Transaction,
        cursor_clause: &str,
        granularity_format: &str,
    ) -> Result<u32, ServerFnError> {
        let query = format!(
            "
        WITH
        all_stats AS (
            SELECT
                strftime('{}', notes.created_at) AS period
            FROM
                kids
            LEFT JOIN notes ON notes.kid_id = kids.id
            WHERE
                kid_id = {}
                {}
            GROUP BY
                period
            ORDER BY
                period DESC
        )
        SELECT
        count(*) periods
        FROM
        all_stats
            ",
            granularity_format, self.kid_id, cursor_clause
        );

        tracing::debug!("SQL get_filter_total: {}", query);

        let row = trx
            .prepare(&query)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .query_row(())
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let total: u32 = row.get(0).map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(total)
    }
}
