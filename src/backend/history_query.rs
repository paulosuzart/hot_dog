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

    pub async fn execute_with_db(&self, conn: &'static libsql::Connection) -> Result<KidHistoryResponse, ServerFnError> {
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
            });
        }

        let current_page = (periods[0].row_num - 1) / self.page_size as u32 + 1;
        let total_pages = (total_count + self.page_size as u32 - 1) / self.page_size as u32;

        let next_cursor = self.extract_next_cursor(&mut periods);
        let kid_history: Vec<KidHistory> = periods.into_iter().map(|r| r.to_kid_history()).collect();

        Ok(KidHistoryResponse {
            history: kid_history,
            cursor: next_cursor,
            granularity: grain_format.to_string(),
            total_count,
            current_page,
            total_pages,
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
    use chrono::{Duration, Utc};
    use libsql::{Builder, Connection};
    use std::boxed::Box;

    /// Creates an in-memory SQLite database with test schema and data
    async fn setup_test_db() -> Connection {
        let db = Builder::new_local(":memory:")
            .build()
            .await
            .expect("Failed to create in-memory database");

        let db_conn = db.connect().expect("Failed to connect to in-memory database");

        // Create schema
        db_conn
            .execute(
                "CREATE TABLE IF NOT EXISTS settings (
                    id integer PRIMARY KEY AUTOINCREMENT,
                    granularity text NOT NULL,
                    created_at text DEFAULT 'datetime(\"now\", \"utc\")'
                )",
                (),
            )
            .await
            .expect("Failed to create settings table");

        db_conn
            .execute(
                "CREATE TABLE IF NOT EXISTS kids (
                    id integer PRIMARY KEY AUTOINCREMENT UNIQUE,
                    name text NOT NULL,
                    created_at text DEFAULT 'datetime(\"now\", \"utc\")'
                )",
                (),
            )
            .await
            .expect("Failed to create kids table");

        db_conn
            .execute(
                "CREATE TABLE IF NOT EXISTS notes (
                    id integer PRIMARY KEY,
                    created_at text DEFAULT 'datetime(''now'', ''utc'')' NOT NULL,
                    quantity integer,
                    kid_id integer NOT NULL,
                    FOREIGN KEY (kid_id) REFERENCES kids(id) ON UPDATE NO ACTION ON DELETE CASCADE
                )",
                (),
            )
            .await
            .expect("Failed to create notes table");

        db_conn
            .execute(
                "CREATE INDEX IF NOT EXISTS kid_id_idx ON notes (kid_id, quantity)",
                (),
            )
            .await
            .expect("Failed to create index");

        // Insert test data
        let now = Utc::now().naive_utc();

        // Kid 1: "Alice" with notes across 3 months
        let alice_name = "Alice";
        db_conn
            .execute(
                "INSERT INTO kids (name, created_at) VALUES (?1, ?2)",
                libsql::params![alice_name, now.format("%Y-%m-%d %H:%M:%S").to_string()],
            )
            .await
            .expect("Failed to insert Alice");

        // Month 3 (most recent): 3 positive notes
        for i in 0..3 {
            let note_time = (now - Duration::days(10 - i))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            db_conn
                .execute(
                    "INSERT INTO notes (kid_id, quantity, created_at) VALUES (1, 1, ?1)",
                    libsql::params![note_time],
                )
                .await
                .expect("Failed to insert Alice note");
        }

        // Month 2: 2 positive, 1 negative
        for i in 0..2 {
            let note_time = (now - Duration::days(30 + i * 3))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            db_conn
                .execute(
                    "INSERT INTO notes (kid_id, quantity, created_at) VALUES (1, 1, ?1)",
                    libsql::params![note_time],
                )
                .await
                .expect("Failed to insert Alice note");
        }
        let note_time = (now - Duration::days(38))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        db_conn
            .execute(
                "INSERT INTO notes (kid_id, quantity, created_at) VALUES (1, -1, ?1)",
                libsql::params![note_time],
            )
            .await
            .expect("Failed to insert Alice note");

        // Month 1 (oldest): 4 positive notes
        for i in 0..4 {
            let note_time = (now - Duration::days(60 + i * 2))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            db_conn
                .execute(
                    "INSERT INTO notes (kid_id, quantity, created_at) VALUES (1, 1, ?1)",
                    libsql::params![note_time],
                )
                .await
                .expect("Failed to insert Alice note");
        }

        // Kid 2: "Bob" with notes across 2 months
        let bob_name = "Bob";
        db_conn
            .execute(
                "INSERT INTO kids (name, created_at) VALUES (?1, ?2)",
                libsql::params![bob_name, now.format("%Y-%m-%d %H:%M:%S").to_string()],
            )
            .await
            .expect("Failed to insert Bob");

        // Month 2 (most recent): 5 positive notes
        for i in 0..5 {
            let note_time = (now - Duration::days(20 - i))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            db_conn
                .execute(
                    "INSERT INTO notes (kid_id, quantity, created_at) VALUES (2, 1, ?1)",
                    libsql::params![note_time],
                )
                .await
                .expect("Failed to insert Bob note");
        }

        // Month 1 (oldest): 2 positive, 2 negative
        for i in 0..2 {
            let note_time = (now - Duration::days(50 + i * 3))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            db_conn
                .execute(
                    "INSERT INTO notes (kid_id, quantity, created_at) VALUES (2, 1, ?1)",
                    libsql::params![note_time],
                )
                .await
                .expect("Failed to insert Bob note");
        }
        for i in 0..2 {
            let note_time = (now - Duration::days(55 + i * 3))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            db_conn
                .execute(
                    "INSERT INTO notes (kid_id, quantity, created_at) VALUES (2, -1, ?1)",
                    libsql::params![note_time],
                )
                .await
                .expect("Failed to insert Bob note");
        }

        db_conn
    }

    // ============================================
    // UNIT TESTS
    // ============================================

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
}
