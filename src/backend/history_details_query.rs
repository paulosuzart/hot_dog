#[cfg(feature = "server")]
use chrono::NaiveDateTime;
#[cfg(feature = "server")]
use dioxus::server::ServerFnError;

#[cfg(feature = "server")]
use crate::backend::kids::Granularity;

#[cfg(feature = "server")]
use crate::models::HistoryDetailResponse;

#[cfg(feature = "server")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryDetailsRepository {
    pub kid_id: u32,
    pub period: String,
    pub granularity: Granularity,
    pub expected_count: u32,
}

#[cfg(feature = "server")]
impl HistoryDetailsRepository {
    pub fn new(kid_id: u32, period: String, granularity: Granularity, expected_count: u32) -> Self {
        Self {
            kid_id,
            period,
            granularity,
            expected_count,
        }
    }

    pub async fn execute_query(
        &self,
        conn: &libsql::Connection,
    ) -> Result<HistoryDetailResponse, ServerFnError> {
        let grain_format = self.granularity.grain_format();
        let grain_value = self
            .granularity
            .grain_value(&self.period)
            .map_err(|e| ServerFnError::new(format!("Failed to parse grain value: {}", e)))?;

        let query = format!(
            "
        SELECT
            kid_id,
            quantity,
            notes.created_at
        FROM
            notes
        WHERE
            kid_id = :kid_id
            AND strftime('{grain_format}', notes.created_at) = :grain_value
        ORDER BY notes.created_at ASC
        "
        );

        tracing::debug!("SQL history_detail: {}", query);

        let stm = conn
            .prepare(&query)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let mut rows = stm
            .query(libsql::named_params! { ":kid_id": self.kid_id, ":grain_value": grain_value })
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let mut results = Vec::new();

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to iterate over rows: {}", e)))?
        {
            use crate::models::NoteDetails;

            let kid_id: u32 = row
                .get(0)
                .map_err(|e| ServerFnError::new(format!("Failed to get kid_id: {}", e)))?;
            let quantity: i32 = row
                .get(1)
                .map_err(|e| ServerFnError::new(format!("Failed to get quantity: {}", e)))?;
            let created_at: String = row
                .get(2)
                .map_err(|e| ServerFnError::new(format!("Failed to get created_at: {}", e)))?;

            let created_at_dt = NaiveDateTime::parse_from_str(&created_at, "%Y-%m-%d %H:%M:%S")
                .map_err(|e| ServerFnError::new(format!("Failed to parse created_at: {}", e)))?;

            results.push(NoteDetails {
                kid_id,
                quantity,
                created_at: created_at_dt,
            });
        }

        Ok(HistoryDetailResponse {
            kid_id: self.kid_id,
            notes: results,
            needs_reload: results.len() != self.expected_count,
        })
    }
}

// ============================================
// TESTS
// ============================================

#[cfg(feature = "server")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::kids::Granularity;
    use crate::backend::test_db::setup_two_period_db;
    use chrono::NaiveDateTime;

    // Fixture: Sam (kid_id=1)
    //   2026-02: Feb 05, Feb 10, Feb 15  — quantity=+1  (3 notes)
    //   2026-01: Jan 10, Jan 20          — quantity=-1  (2 notes)

    #[tokio::test]
    async fn test_returns_all_notes_for_monthly_period() {
        let conn = setup_two_period_db().await;

        let q = HistoryDetailsRepository::new(1, "2026-02".to_string(), Granularity::Monthly);
        let result = q.execute_query(&conn).await.unwrap();

        assert_eq!(result.kid_id, 1);
        assert_eq!(result.notes.len(), 3, "2026-02 must have 3 notes");
        assert!(result.notes.iter().all(|n| n.quantity == 1));
    }

    #[tokio::test]
    async fn test_returns_notes_for_older_monthly_period() {
        let conn = setup_two_period_db().await;

        let q = HistoryDetailsRepository::new(1, "2026-01".to_string(), Granularity::Monthly);
        let result = q.execute_query(&conn).await.unwrap();

        assert_eq!(result.notes.len(), 2, "2026-01 must have 2 notes");
        assert!(result.notes.iter().all(|n| n.quantity == -1));
    }

    #[tokio::test]
    async fn test_returns_empty_for_period_with_no_notes() {
        let conn = setup_two_period_db().await;

        let q = HistoryDetailsRepository::new(1, "2025-12".to_string(), Granularity::Monthly);
        let result = q.execute_query(&conn).await.unwrap();

        assert_eq!(result.notes.len(), 0);
    }

    #[tokio::test]
    async fn test_returns_empty_for_unknown_kid() {
        let conn = setup_two_period_db().await;

        let q = HistoryDetailsRepository::new(999, "2026-02".to_string(), Granularity::Monthly);
        let result = q.execute_query(&conn).await.unwrap();

        assert_eq!(result.notes.len(), 0);
        assert_eq!(result.kid_id, 999);
    }

    #[tokio::test]
    async fn test_note_kid_id_matches_query() {
        let conn = setup_two_period_db().await;

        let q = HistoryDetailsRepository::new(1, "2026-02".to_string(), Granularity::Monthly);
        let result = q.execute_query(&conn).await.unwrap();

        assert!(result.notes.iter().all(|n| n.kid_id == 1));
    }

    #[tokio::test]
    async fn test_notes_are_ordered_by_created_at_ascending() {
        let conn = setup_two_period_db().await;

        let q = HistoryDetailsRepository::new(1, "2026-02".to_string(), Granularity::Monthly);
        let result = q.execute_query(&conn).await.unwrap();

        let dates: Vec<NaiveDateTime> = result.notes.iter().map(|n| n.created_at).collect();
        let mut sorted = dates.clone();
        sorted.sort();
        assert_eq!(dates, sorted, "notes must be ordered oldest-first");
    }

    #[tokio::test]
    async fn test_created_at_timestamps_are_correct() {
        let conn = setup_two_period_db().await;

        let q = HistoryDetailsRepository::new(1, "2026-02".to_string(), Granularity::Monthly);
        let result = q.execute_query(&conn).await.unwrap();

        let expected = [
            NaiveDateTime::parse_from_str("2026-02-05 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            NaiveDateTime::parse_from_str("2026-02-10 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            NaiveDateTime::parse_from_str("2026-02-15 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        ];
        let actual: Vec<NaiveDateTime> = result.notes.iter().map(|n| n.created_at).collect();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_daily_granularity_returns_single_day() {
        let conn = setup_two_period_db().await;

        // Only the Feb-10 note matches this exact day.
        let q = HistoryDetailsRepository::new(1, "2026-02-10".to_string(), Granularity::Daily);
        let result = q.execute_query(&conn).await.unwrap();

        assert_eq!(result.notes.len(), 1);
        assert_eq!(result.notes[0].quantity, 1);
        assert_eq!(
            result.notes[0].created_at,
            NaiveDateTime::parse_from_str("2026-02-10 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
    }

    #[tokio::test]
    async fn test_daily_granularity_returns_empty_for_day_without_notes() {
        let conn = setup_two_period_db().await;

        let q = HistoryDetailsRepository::new(1, "2026-02-01".to_string(), Granularity::Daily);
        let result = q.execute_query(&conn).await.unwrap();

        assert_eq!(result.notes.len(), 0);
    }
}
