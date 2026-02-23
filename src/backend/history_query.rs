#[cfg(feature = "server")]
use dioxus::server::ServerFnError;

#[cfg(feature = "server")]
use base64::{engine::general_purpose::URL_SAFE, Engine as _};

#[cfg(feature = "server")]
use crate::{
    backend::constants::MAX_HISTORY_PAGE_SIZE,
    backend::kids::{Cursor, Granularity},
    models::{KidHistory, KidHistoryResponse},
};

#[cfg(feature = "server")]
#[derive(Debug, serde::Deserialize)]
struct NumberedHistoryRow {
    kid_id: u32,
    period: String,
    total: i32,
    result: i32,
    neg_count: i32,
    post_count: i32,
    name: String,
    row_num: u32,
    total_count: u32,
}

#[cfg(feature = "server")]
impl NumberedHistoryRow {
    fn to_kid_history(self) -> KidHistory {
        KidHistory {
            id: self.kid_id,
            period: self.period,
            total: self.total,
            result: self.result,
            neg_count: self.neg_count,
            post_count: self.post_count,
            name: self.name,
        }
    }
}

#[cfg(feature = "server")]
pub struct KidHistoryQuery {
    kid_id: u32,
    cursor: Option<Cursor>,
    page_size: u8,
    granularity: Granularity,
}

#[cfg(feature = "server")]
impl KidHistoryQuery {
    pub fn new(
        kid_id: u32,
        cursor: Option<Cursor>,
        page_size: u8,
        granularity: Granularity,
    ) -> Self {
        let final_page_size = std::cmp::min(page_size, *MAX_HISTORY_PAGE_SIZE);
        if final_page_size != page_size {
            tracing::warn!(
                "Using default page size. Request {} exceeded system max {}",
                page_size,
                *MAX_HISTORY_PAGE_SIZE
            );
        }

        Self {
            kid_id,
            cursor: cursor,
            page_size: final_page_size,
            granularity,
        }
    }

    fn cursor_clause(&self) -> String {
        if let Some(cursor) = &self.cursor {
            format!(
                " AND strftime('{}', notes.created_at) > '{}' ",
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
        self.execute_with_db(conn).await
    }

    pub async fn execute_with_db(
        &self,
        conn: &'static libsql::Connection,
    ) -> Result<KidHistoryResponse, ServerFnError> {
        let cursor_clause = self.cursor_clause();
        let grain_format = self.granularity.grain_format();

        let (mut periods, total_count) = self.fetch_periods(conn, &cursor_clause).await?;

        if periods.is_empty() {
            return Ok(KidHistoryResponse {
                history: vec![],
                cursor: None,
                granularity: grain_format.to_string(),
                total_count: 0,
                current_page: 1,
                total_pages: 0,
                name: "".to_string(),
            });
        }

        let current_page = (periods[0].row_num - 1) / self.page_size as u32 + 1;
        let total_pages = (total_count + self.page_size as u32 - 1) / self.page_size as u32;

        let next_cursor = self.extract_next_cursor(&mut periods);
        let kid_history: Vec<KidHistory> =
            periods.into_iter().map(|r| r.to_kid_history()).collect();

        let kid_name = kid_history[0].name.clone();
        Ok(KidHistoryResponse {
            history: kid_history,
            cursor: next_cursor,
            granularity: grain_format.to_string(),
            total_count,
            current_page,
            total_pages,
            name: kid_name,
        })
    }

    /// Extracts the next cursor from the periods list if there are more items than the page size.
    fn extract_next_cursor(&self, periods: &mut Vec<NumberedHistoryRow>) -> Option<String> {
        if periods.len() > self.page_size as usize {
            // Discard the extra "peek" item — it only proves a next page exists.
            periods.pop();
            // Cursor points to the LAST item of the current page so the next
            // query can filter `row_num > cursor_row_num` correctly.
            let last_in_page = periods.last().unwrap();
            let cursor_str = format!("{}|{}", last_in_page.period, self.granularity);
            Some(URL_SAFE.encode(cursor_str.as_bytes()))
        } else {
            None
        }
    }

    /// Fetches the periods for the given kid_id, cursor and granularity with window functions.
    async fn fetch_periods(
        &self,
        conn: &'static libsql::Connection,
        _cursor_clause: &str,
    ) -> Result<(Vec<NumberedHistoryRow>, u32), ServerFnError> {
        let grain_format = self.granularity.grain_format();
        let query = format!(
            "
            WITH all_periods AS (
                SELECT
                    period,
                    total,
                    neg_count,
                    post_count,
                    kid_id,
                    name,
                    ROW_NUMBER() OVER (ORDER BY period DESC) as row_num
                FROM (
                    SELECT
                        strftime('{grain_format}', notes.created_at) AS period,
                        SUM(quantity) AS total,
                        COUNT(CASE WHEN quantity = -1 THEN 1 END) neg_count,
                        COUNT(CASE WHEN quantity = 1 THEN 1 END) post_count,
                        kids.id AS kid_id,
                        kids.name AS name
                    FROM kids
                    LEFT JOIN notes ON notes.kid_id = kids.id
                    WHERE kid_id = :kid_id
                    GROUP BY period
                    ORDER BY period DESC
                )
            ),
            total_counts AS (
                SELECT COUNT(*) as total_count FROM all_periods
            ),
            cursor_row AS (
                SELECT MAX(row_num) as max_row_num FROM (
                    SELECT
                        ROW_NUMBER() OVER (ORDER BY period DESC) as row_num,
                        period
                    FROM (
                        SELECT strftime('{grain_format}', notes.created_at) AS period
                        FROM kids
                        LEFT JOIN notes ON notes.kid_id = kids.id
                        WHERE kid_id = :kid_id
                        GROUP BY period
                        ORDER BY period DESC
                    )
                )
                WHERE period = :cursor_value
            )
            SELECT
                p.period,
                p.total,
                p.neg_count,
                p.post_count,
                p.kid_id,
                p.name,
                p.row_num,
                t.total_count,
                (p.neg_count + p.post_count) as result
            FROM all_periods p
            CROSS JOIN total_counts t
            CROSS JOIN cursor_row c
            WHERE p.row_num > COALESCE(c.max_row_num, 0)
            ORDER BY p.period DESC
            LIMIT :limit
        "
        );

        tracing::debug!("SQL fetch_periods: {}", query);

        let cursor_value = match &self.cursor {
            Some(c) => c.grain_value.clone(),
            None => "".to_string(),
        };

        let stm = conn
            .prepare(&query)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let mut rows = stm
            .query(libsql::named_params! {
                ":limit": self.page_size + 1,
                ":kid_id": self.kid_id,
                ":cursor_value": cursor_value
            })
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let mut numbered_history: Vec<NumberedHistoryRow> = Vec::new();
        let mut total_count = 0u32;

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
        {
            let row_data = libsql::de::from_row::<NumberedHistoryRow>(&row)
                .map_err(|e| ServerFnError::new(e.to_string()))?;

            if numbered_history.is_empty() {
                total_count = row_data.total_count;
            }

            numbered_history.push(row_data);
        }

        Ok((numbered_history, total_count))
    }
}

// ============================================
// TESTS
// ============================================

// TESTS
// ============================================

#[cfg(feature = "server")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::test_db::{setup_test_db, setup_two_period_db};
    use chrono::{Duration, Utc};
    use std::boxed::Box;
    use std::str::FromStr;

    // ============================================
    // UNIT TESTS
    // ============================================

    #[test]
    fn test_new_create_query_with_correct_page_size() {
        let exceeding_page_size = *MAX_HISTORY_PAGE_SIZE + 1;
        let granularity = Granularity::Daily;
        let query = KidHistoryQuery::new(1, None, exceeding_page_size, granularity);

        assert_eq!(query.kid_id, 1);
        assert!(query.cursor.is_none());
        assert_eq!(query.page_size, *MAX_HISTORY_PAGE_SIZE);
        assert_eq!(query.granularity, Granularity::Daily);
    }

    #[test]
    fn test_new_creates_query() {
        let granularity = Granularity::Daily;
        let query = KidHistoryQuery::new(1, None, 10, granularity);

        assert_eq!(query.kid_id, 1);
        assert!(query.cursor.is_none());
        assert_eq!(query.page_size, 10);
        assert_eq!(query.granularity, Granularity::Daily);
    }

    #[test]
    fn test_new_with_cursor() {
        let granularity = Granularity::Daily;
        let cursor = Cursor {
            grain_value: "2024-01-15".to_string(),
            grain_format: "%Y-%m-%d",
            granularity: Granularity::Daily,
        };
        let query = KidHistoryQuery::new(1, Some(cursor), 10, granularity);

        assert_eq!(query.kid_id, 1);
        assert!(query.cursor.is_some());
        assert_eq!(query.page_size, 10);
        assert_eq!(query.granularity, Granularity::Daily);
    }

    #[test]
    fn test_cursor_clause_without_cursor() {
        let query = KidHistoryQuery::new(1, None, 10, Granularity::Daily);
        let clause = query.cursor_clause();

        assert_eq!(clause, "");
    }

    #[test]
    fn test_cursor_clause_with_cursor() {
        let cursor = Cursor {
            grain_value: "2024-01-15".to_string(),
            grain_format: "%Y-%m-%d",
            granularity: Granularity::Daily,
        };
        let query = KidHistoryQuery::new(1, Some(cursor), 10, Granularity::Daily);
        let clause = query.cursor_clause();

        assert!(clause.contains(" > "));
        assert!(clause.contains("2024-01-15"));
        assert!(clause.contains("%Y-%m-%d"));
    }

    #[test]
    fn test_cursor_clause_with_weekly_granularity() {
        let cursor = Cursor {
            grain_value: "2024-W03".to_string(),
            grain_format: "%Y-W%W",
            granularity: Granularity::Weekly,
        };
        let query = KidHistoryQuery::new(1, Some(cursor), 10, Granularity::Weekly);
        let clause = query.cursor_clause();

        assert!(clause.contains("%Y-W%W"));
        assert!(clause.contains("2024-W03"));
    }

    #[test]
    fn test_extract_next_cursor_exact_page_size() {
        let query = KidHistoryQuery::new(1, None, 3, Granularity::Daily);
        let mut periods = vec![
            NumberedHistoryRow {
                kid_id: 1,
                period: "2024-01-03".to_string(),
                total: 5,
                result: 3,
                neg_count: 0,
                post_count: 3,
                name: "Alice".to_string(),
                row_num: 1,
                total_count: 3,
            },
            NumberedHistoryRow {
                kid_id: 1,
                period: "2024-01-02".to_string(),
                total: 2,
                result: 1,
                neg_count: 0,
                post_count: 1,
                name: "Alice".to_string(),
                row_num: 2,
                total_count: 3,
            },
            NumberedHistoryRow {
                kid_id: 1,
                period: "2024-01-01".to_string(),
                total: 4,
                result: 4,
                neg_count: 0,
                post_count: 4,
                name: "Alice".to_string(),
                row_num: 3,
                total_count: 3,
            },
        ];

        let next_cursor = query.extract_next_cursor(&mut periods);

        assert!(next_cursor.is_none());
        assert_eq!(periods.len(), 3);
    }

    #[test]
    fn test_extract_next_cursor_more_than_page_size() {
        let query = KidHistoryQuery::new(1, None, 2, Granularity::Daily);
        let mut periods = vec![
            NumberedHistoryRow {
                kid_id: 1,
                period: "2024-01-03".to_string(),
                total: 5,
                result: 3,
                neg_count: 0,
                post_count: 3,
                name: "Alice".to_string(),
                row_num: 1,
                total_count: 3,
            },
            NumberedHistoryRow {
                kid_id: 1,
                period: "2024-01-02".to_string(),
                total: 2,
                result: 1,
                neg_count: 0,
                post_count: 1,
                name: "Alice".to_string(),
                row_num: 2,
                total_count: 3,
            },
            NumberedHistoryRow {
                kid_id: 1,
                period: "2024-01-01".to_string(),
                total: 4,
                result: 4,
                neg_count: 0,
                post_count: 4,
                name: "Alice".to_string(),
                row_num: 3,
                total_count: 3,
            },
        ];

        let next_cursor = query.extract_next_cursor(&mut periods);

        assert!(next_cursor.is_some());
        assert_eq!(periods.len(), 2);
        assert_eq!(periods[0].period, "2024-01-03");
        assert_eq!(periods[1].period, "2024-01-02");
    }

    // ============================================
    // INTEGRATION TESTS
    // ============================================

    #[tokio::test]
    async fn test_execute_returns_empty_history_for_no_data() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let query = KidHistoryQuery::new(999, None, 10, Granularity::Daily);
        let result = query.execute_with_db(db_static).await.unwrap();

        assert_eq!(result.history.len(), 0);
        assert_eq!(result.total_count, 0);
        assert_eq!(result.current_page, 1);
        assert_eq!(result.total_pages, 0);
        assert!(result.cursor.is_none());
    }

    #[tokio::test]
    async fn test_execute_paginates_correctly() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let query = KidHistoryQuery::new(1, None, 2, Granularity::Monthly);
        let result = query.execute_with_db(db_static).await.unwrap();

        assert_eq!(result.history.len(), 2);
        assert_eq!(result.total_count, 3);
        assert_eq!(result.current_page, 1);
        assert_eq!(result.total_pages, 2);
        assert!(result.cursor.is_some());
    }

    #[tokio::test]
    async fn test_execute_with_cursor_pagination() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let now = Utc::now().naive_utc();
        let middle_month = (now - Duration::days(30)).format("%Y-%m").to_string();
        let cursor = Cursor {
            grain_value: middle_month,
            grain_format: "%Y-%m",
            granularity: Granularity::Monthly,
        };

        let query = KidHistoryQuery::new(1, Some(cursor), 10, Granularity::Monthly);
        let result = query.execute_with_db(db_static).await.unwrap();

        assert_eq!(result.history.len(), 1);
        assert_eq!(result.total_count, 3);
        assert_eq!(result.current_page, 1);
        assert_eq!(result.total_pages, 1);
        assert!(result.cursor.is_none());
    }

    #[tokio::test]
    async fn test_execute_counts_total_correctly() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let query = KidHistoryQuery::new(1, None, 10, Granularity::Monthly);
        let result = query.execute_with_db(db_static).await.unwrap();

        assert_eq!(result.total_count, 3);
        assert_eq!(result.total_pages, 1);
        assert_eq!(result.current_page, 1);
    }

    #[tokio::test]
    async fn test_execute_returns_correct_kid_data() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let query = KidHistoryQuery::new(1, None, 10, Granularity::Monthly);
        let result = query.execute_with_db(db_static).await.unwrap();

        assert_eq!(result.history.len(), 3);
        assert_eq!(result.history[0].name, "Alice");
        assert_eq!(result.history[0].id, 1);
        assert_eq!(result.current_page, 1);
        assert_eq!(result.total_pages, 1);
    }

    #[tokio::test]
    async fn test_execute_calculates_correct_counts() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let query = KidHistoryQuery::new(1, None, 10, Granularity::Monthly);
        let result = query.execute_with_db(db_static).await.unwrap();

        let middle_month = &result.history[1];
        assert_eq!(middle_month.total, 1);
        assert_eq!(middle_month.neg_count, 1);
        assert_eq!(middle_month.post_count, 2);
        assert_eq!(middle_month.result, 3);
    }

    #[tokio::test]
    async fn test_execute_calculates_pagination_from_row_number() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let query = KidHistoryQuery::new(1, None, 1, Granularity::Monthly);
        let result = query.execute_with_db(db_static).await.unwrap();

        assert_eq!(result.history.len(), 1);
        assert_eq!(result.current_page, 1);
        assert_eq!(result.total_pages, 3);
        assert!(result.cursor.is_some());
    }

    #[tokio::test]
    async fn test_execute_second_page_correctly_identified() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let now = Utc::now().naive_utc();
        let first_month = (now - Duration::days(10)).format("%Y-%m").to_string();
        let cursor = Cursor {
            grain_value: first_month,
            grain_format: "%Y-%m",
            granularity: Granularity::Monthly,
        };

        let query = KidHistoryQuery::new(1, Some(cursor), 1, Granularity::Monthly);
        let result = query.execute_with_db(db_static).await.unwrap();

        assert_eq!(result.history.len(), 1);
        assert_eq!(result.current_page, 2);
        assert_eq!(result.total_pages, 3);
        assert!(result.cursor.is_some());
    }

    /// Regression: when exactly 2 periods exist and page_size=1, clicking Next must
    /// show page 2 (not an empty result).  Previously the cursor pointed at the extra
    /// peek item, so page 2 queried `row_num > 2` and got nothing.
    #[tokio::test]
    async fn test_two_period_second_page_not_empty() {
        let db = setup_two_period_db().await;
        let db_static = Box::leak(Box::new(db));

        // Sam (kid_id=1) has notes in exactly 2 months.
        let query1 = KidHistoryQuery::new(1, None, 1, Granularity::Monthly);
        let page1 = query1.execute_with_db(db_static).await.unwrap();

        assert_eq!(page1.history.len(), 1, "page 1 should have 1 item");
        assert_eq!(page1.current_page, 1);
        assert_eq!(page1.total_pages, 2);
        assert!(
            page1.cursor.is_some(),
            "page 1 must expose a next-page cursor"
        );

        // Build page-2 query from the encoded cursor returned by page 1.
        let cursor2 = Cursor::from_str(page1.cursor.as_ref().unwrap())
            .expect("cursor from page 1 must parse");
        let query2 = KidHistoryQuery::new(1, Some(cursor2), 1, Granularity::Monthly);
        let page2 = query2.execute_with_db(db_static).await.unwrap();

        assert_eq!(page2.history.len(), 1, "page 2 must not be empty");
        assert_eq!(page2.current_page, 2);
        assert_eq!(page2.total_pages, 2);
        assert!(
            page2.cursor.is_none(),
            "page 2 is the last page — no cursor expected"
        );
        assert_ne!(
            page1.history[0].period, page2.history[0].period,
            "pages must show different periods"
        );
    }

    /// Navigate all three monthly pages for Alice using only the cursors returned by
    /// each response — i.e. no manually constructed Cursor objects.
    #[tokio::test]
    async fn test_full_three_page_navigation() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        // Page 1
        let q1 = KidHistoryQuery::new(1, None, 1, Granularity::Monthly);
        let p1 = q1.execute_with_db(db_static).await.unwrap();

        assert_eq!(p1.history.len(), 1);
        assert_eq!(p1.current_page, 1);
        assert_eq!(p1.total_pages, 3);
        assert!(p1.cursor.is_some());

        // Page 2 — built from the cursor returned on page 1
        let c2 = Cursor::from_str(p1.cursor.as_ref().unwrap()).expect("page-1 cursor must parse");
        let q2 = KidHistoryQuery::new(1, Some(c2), 1, Granularity::Monthly);
        let p2 = q2.execute_with_db(db_static).await.unwrap();

        assert_eq!(p2.history.len(), 1);
        assert_eq!(p2.current_page, 2);
        assert_eq!(p2.total_pages, 3);
        assert!(p2.cursor.is_some());
        assert_ne!(p1.history[0].period, p2.history[0].period);

        // Page 3 — built from the cursor returned on page 2
        let c3 = Cursor::from_str(p2.cursor.as_ref().unwrap()).expect("page-2 cursor must parse");
        let q3 = KidHistoryQuery::new(1, Some(c3), 1, Granularity::Monthly);
        let p3 = q3.execute_with_db(db_static).await.unwrap();

        assert_eq!(p3.history.len(), 1);
        assert_eq!(p3.current_page, 3);
        assert_eq!(p3.total_pages, 3);
        assert!(p3.cursor.is_none(), "last page must have no cursor");
        assert_ne!(p2.history[0].period, p3.history[0].period);

        // All three periods must be distinct
        let periods: std::collections::HashSet<_> = [
            &p1.history[0].period,
            &p2.history[0].period,
            &p3.history[0].period,
        ]
        .into_iter()
        .collect();
        assert_eq!(periods.len(), 3, "each page must show a unique period");
    }
}
