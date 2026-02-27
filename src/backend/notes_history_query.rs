#[cfg(feature = "server")]
use dioxus::server::ServerFnError;

#[cfg(feature = "server")]
use base64::{engine::general_purpose::URL_SAFE, Engine as _};

#[cfg(feature = "server")]
use crate::{
    backend::constants::MAX_NOTES_HISTORY_PAGE_SIZE,
    models::{NoteHistory, NoteHistoryResponse},
};

#[cfg(feature = "server")]
#[derive(Debug, serde::Deserialize)]
struct NumberedNoteRow {
    id: i64,
    kid_id: u32,
    kid_name: String,
    quantity: i32,
    created_at: String,
    row_num: u32,
    total_count: u32,
}

#[cfg(feature = "server")]
impl NumberedNoteRow {
    fn to_note_history(self) -> NoteHistory {
        NoteHistory {
            id: self.id,
            kid_id: self.kid_id,
            kid_name: self.kid_name,
            quantity: self.quantity,
            created_at: self.created_at,
        }
    }
}

/// Cursor for notes history pagination - carries all context needed for the next page.
/// When provided, filter params are ignored and all values come from the cursor.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NotesHistoryCursor {
    pub row_num: u32,
    pub kid_id: Option<u32>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub sort_by: String,
    pub sort_order: String,
    pub page_size: u8,
}

/// Filter parameters for notes history queries (used only for the first page)
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NotesHistoryFilter {
    pub kid_id: Option<u32>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub cursor: Option<String>,
    pub page_size: u8,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[cfg(feature = "server")]
pub struct NotesHistoryQuery {
    kid_id: Option<u32>,
    date_from: Option<String>,
    date_to: Option<String>,
    cursor_row_num: u32,
    page_size: u8,
    sort_by: String,
    sort_order: String,
}

#[cfg(feature = "server")]
impl NotesHistoryQuery {
    pub fn new(filter: NotesHistoryFilter) -> Self {
        // If cursor is provided, decode it and use ALL values from cursor (ignore filter params)
        // If no cursor, use the filter params for the first page
        if let Some(cursor_str) = &filter.cursor {
            match Self::decode_cursor(cursor_str) {
                Ok(cursor) => {
                    return Self {
                        kid_id: cursor.kid_id,
                        date_from: cursor.date_from,
                        date_to: cursor.date_to,
                        cursor_row_num: cursor.row_num,
                        page_size: cursor.page_size,
                        sort_by: cursor.sort_by,
                        sort_order: cursor.sort_order,
                    };
                }
                Err(e) => {
                    tracing::warn!("Failed to decode cursor, falling back to filter params: {}", e);
                }
            }
        }

        // No cursor (or failed decode) - use filter params for first page
        let final_page_size = std::cmp::min(filter.page_size, *MAX_NOTES_HISTORY_PAGE_SIZE);
        if final_page_size != filter.page_size && filter.page_size > 0 {
            tracing::warn!(
                "Using default page size. Request {} exceeded system max {}",
                filter.page_size,
                *MAX_NOTES_HISTORY_PAGE_SIZE
            );
        }

        let sort_by = filter.sort_by.unwrap_or_else(|| "created_at".to_string());
        let sort_order = filter.sort_order.unwrap_or_else(|| "desc".to_string());

        Self {
            kid_id: filter.kid_id,
            date_from: filter.date_from,
            date_to: filter.date_to,
            cursor_row_num: 0,
            page_size: final_page_size,
            sort_by,
            sort_order,
        }
    }

    fn decode_cursor(cursor_str: &str) -> Result<NotesHistoryCursor, ServerFnError> {
        let decoded = URL_SAFE
            .decode(cursor_str.as_bytes())
            .map_err(|e| ServerFnError::new(format!("Failed to decode cursor: {}", e)))?;
        let json_str = String::from_utf8(decoded)
            .map_err(|e| ServerFnError::new(format!("Invalid UTF-8 in cursor: {}", e)))?;
        Ok(serde_json::from_str::<NotesHistoryCursor>(&json_str)
            .map_err(|e| ServerFnError::new(format!("Invalid cursor JSON: {}", e)))?)
    }

    fn encode_cursor(&self, row_num: u32) -> String {
        let cursor = NotesHistoryCursor {
            row_num,
            kid_id: self.kid_id,
            date_from: self.date_from.clone(),
            date_to: self.date_to.clone(),
            sort_by: self.sort_by.clone(),
            sort_order: self.sort_order.clone(),
            page_size: self.page_size,
        };
        let json = serde_json::to_string(&cursor).unwrap();
        URL_SAFE.encode(json.as_bytes())
    }

    fn get_sort_clause_for_row_number(&self) -> String {
        let sort_column = match self.sort_by.as_str() {
            "kid_name" => "kids.name",
            "created_at" | _ => "notes.created_at",
        };
        let sort_dir = if self.sort_order == "asc" { "ASC" } else { "DESC" };
        format!("ORDER BY {} {}", sort_column, sort_dir)
    }

    fn get_sort_clause_for_outer_query(&self) -> String {
        let sort_column = match self.sort_by.as_str() {
            "kid_name" => "n.kid_name",
            "created_at" | _ => "n.created_at",
        };
        let sort_dir = if self.sort_order == "asc" { "ASC" } else { "DESC" };
        format!("ORDER BY {} {}", sort_column, sort_dir)
    }

    pub async fn execute(&self) -> Result<NoteHistoryResponse, ServerFnError> {
        use crate::backend::turso::get_db;

        let conn = get_db().await;
        self.execute_with_db(conn).await
    }

    pub async fn execute_with_db(
        &self,
        conn: &'static libsql::Connection,
    ) -> Result<NoteHistoryResponse, ServerFnError> {
        let (mut notes, total_count) = self.fetch_notes(conn).await?;

        if notes.is_empty() {
            return Ok(NoteHistoryResponse {
                notes: vec![],
                cursor: None,
                total_count: 0,
                current_page: 1,
                total_pages: 0,
            });
        }

        let current_page = (notes[0].row_num - 1) / self.page_size as u32 + 1;
        let total_pages = (total_count + self.page_size as u32 - 1) / self.page_size as u32;

        let next_cursor = self.extract_next_cursor(&mut notes);
        let note_history: Vec<NoteHistory> =
            notes.into_iter().map(|r| r.to_note_history()).collect();

        Ok(NoteHistoryResponse {
            notes: note_history,
            cursor: next_cursor,
            total_count,
            current_page,
            total_pages,
        })
    }

    /// Extracts the next cursor from the notes list if there are more items than the page size.
    fn extract_next_cursor(&self, notes: &mut Vec<NumberedNoteRow>) -> Option<String> {
        if notes.len() > self.page_size as usize {
            // Discard the extra "peek" item
            notes.pop();
            // Cursor encodes all context needed for the next page
            let last_in_page = notes.last().unwrap();
            Some(self.encode_cursor(last_in_page.row_num))
        } else {
            None
        }
    }

    /// Fetches notes with filtering, sorting, and cursor-based pagination.
    async fn fetch_notes(
        &self,
        conn: &'static libsql::Connection,
    ) -> Result<(Vec<NumberedNoteRow>, u32), ServerFnError> {
        let sort_clause_inner = self.get_sort_clause_for_row_number();
        let sort_clause_outer = self.get_sort_clause_for_outer_query();

        let query = format!(
            r#"
            WITH all_notes AS (
                SELECT
                    notes.id,
                    notes.kid_id,
                    kids.name as kid_name,
                    notes.quantity,
                    notes.created_at,
                    ROW_NUMBER() OVER ({sort_clause_inner}) as row_num
                FROM notes
                JOIN kids ON notes.kid_id = kids.id
                WHERE 1=1
                    AND (:kid_id IS NULL OR notes.kid_id = :kid_id)
                    AND (:date_from IS NULL OR notes.created_at >= :date_from)
                    AND (:date_to IS NULL OR notes.created_at <= :date_to)
            ),
            total_counts AS (
                SELECT COUNT(*) as total_count FROM all_notes
            )
            SELECT
                n.id,
                n.kid_id,
                n.kid_name,
                n.quantity,
                n.created_at,
                n.row_num,
                t.total_count
            FROM all_notes n
            CROSS JOIN total_counts t
            WHERE n.row_num > :cursor_row_num
            {sort_clause_outer}
            LIMIT :limit
            "#
        );

        tracing::debug!("SQL fetch_notes: {}", query);

        let date_from = self.date_from.clone().unwrap_or_default();
        let date_to = self.date_to.clone().unwrap_or_default();

        let stm = conn
            .prepare(&query)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let kid_id_param = self.kid_id.map(|id| id as i64);
        let date_from_param = if date_from.is_empty() { None } else { Some(date_from) };
        let date_to_param = if date_to.is_empty() { None } else { Some(date_to) };

        let mut rows = stm
            .query(libsql::named_params! {
                ":limit": self.page_size + 1,
                ":kid_id": kid_id_param,
                ":date_from": date_from_param,
                ":date_to": date_to_param,
                ":cursor_row_num": self.cursor_row_num,
            })
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

        let mut numbered_notes: Vec<NumberedNoteRow> = Vec::new();
        let mut total_count = 0u32;

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
        {
            let row_data = libsql::de::from_row::<NumberedNoteRow>(&row)
                .map_err(|e| ServerFnError::new(e.to_string()))?;

            if numbered_notes.is_empty() {
                total_count = row_data.total_count;
            }

            numbered_notes.push(row_data);
        }

        Ok((numbered_notes, total_count))
    }
}

// ============================================
// TESTS
// ============================================

#[cfg(feature = "server")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_query_with_defaults() {
        let filter = NotesHistoryFilter::default();
        let query = NotesHistoryQuery::new(filter);

        assert!(query.kid_id.is_none());
        assert!(query.date_from.is_none());
        assert!(query.date_to.is_none());
        assert_eq!(query.cursor_row_num, 0);
        assert_eq!(query.page_size, 0); // default u8 is 0
        assert_eq!(query.sort_by, "created_at");
        assert_eq!(query.sort_order, "desc");
    }

    #[test]
    fn test_new_with_all_filter_params() {
        let filter = NotesHistoryFilter {
            kid_id: Some(1),
            date_from: Some("2026-01-01".to_string()),
            date_to: Some("2026-12-31".to_string()),
            cursor: None,
            page_size: 20,
            sort_by: Some("kid_name".to_string()),
            sort_order: Some("asc".to_string()),
        };
        let query = NotesHistoryQuery::new(filter);

        assert_eq!(query.kid_id, Some(1));
        assert_eq!(query.date_from, Some("2026-01-01".to_string()));
        assert_eq!(query.date_to, Some("2026-12-31".to_string()));
        assert_eq!(query.cursor_row_num, 0);
        assert_eq!(query.page_size, 20);
        assert_eq!(query.sort_by, "kid_name");
        assert_eq!(query.sort_order, "asc");
    }

    #[test]
    fn test_new_uses_cursor_over_filter_params() {
        // Create a cursor with different values than the filter
        let cursor = NotesHistoryCursor {
            row_num: 50,
            kid_id: Some(2),
            date_from: Some("2025-01-01".to_string()),
            date_to: Some("2025-12-31".to_string()),
            sort_by: "kid_name".to_string(),
            sort_order: "asc".to_string(),
            page_size: 15,
        };
        let json = serde_json::to_string(&cursor).unwrap();
        let cursor_str = URL_SAFE.encode(json.as_bytes());

        // Filter has different values, but they should be ignored
        let filter = NotesHistoryFilter {
            kid_id: Some(1),
            date_from: Some("2026-01-01".to_string()),
            date_to: Some("2026-12-31".to_string()),
            cursor: Some(cursor_str),
            page_size: 20,
            sort_by: Some("created_at".to_string()),
            sort_order: Some("desc".to_string()),
        };
        let query = NotesHistoryQuery::new(filter);

        // Values should come from cursor, not filter
        assert_eq!(query.kid_id, Some(2));
        assert_eq!(query.date_from, Some("2025-01-01".to_string()));
        assert_eq!(query.date_to, Some("2025-12-31".to_string()));
        assert_eq!(query.cursor_row_num, 50);
        assert_eq!(query.page_size, 15);
        assert_eq!(query.sort_by, "kid_name");
        assert_eq!(query.sort_order, "asc");
    }

    #[test]
    fn test_get_sort_clause_created_at_desc() {
        let filter = NotesHistoryFilter::default();
        let query = NotesHistoryQuery::new(filter);
        let clause = query.get_sort_clause_for_row_number();

        assert!(clause.contains("notes.created_at"));
        assert!(clause.contains("DESC"));
    }

    #[test]
    fn test_get_sort_clause_kid_name_asc() {
        let filter = NotesHistoryFilter {
            sort_by: Some("kid_name".to_string()),
            sort_order: Some("asc".to_string()),
            ..Default::default()
        };
        let query = NotesHistoryQuery::new(filter);
        let clause = query.get_sort_clause_for_row_number();

        assert!(clause.contains("kids.name"));
        assert!(clause.contains("ASC"));
    }

    #[test]
    fn test_encode_decode_cursor_roundtrip() {
        let cursor = NotesHistoryCursor {
            row_num: 42,
            kid_id: Some(3),
            date_from: Some("2026-01-15".to_string()),
            date_to: Some("2026-02-20".to_string()),
            sort_by: "kid_name".to_string(),
            sort_order: "asc".to_string(),
            page_size: 25,
        };
        
        let json = serde_json::to_string(&cursor).unwrap();
        let encoded = URL_SAFE.encode(json.as_bytes());
        
        let decoded = NotesHistoryQuery::decode_cursor(&encoded).unwrap();
        
        assert_eq!(decoded.row_num, 42);
        assert_eq!(decoded.kid_id, Some(3));
        assert_eq!(decoded.date_from, Some("2026-01-15".to_string()));
        assert_eq!(decoded.date_to, Some("2026-02-20".to_string()));
        assert_eq!(decoded.sort_by, "kid_name");
        assert_eq!(decoded.sort_order, "asc");
        assert_eq!(decoded.page_size, 25);
    }

    #[tokio::test]
    async fn test_execute_returns_empty_for_no_data() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let filter = NotesHistoryFilter {
            kid_id: Some(999),
            page_size: 10,
            ..Default::default()
        };
        let query = NotesHistoryQuery::new(filter);
        let result = query.execute_with_db(db_static).await.unwrap();

        assert_eq!(result.notes.len(), 0);
        assert_eq!(result.total_count, 0);
    }

    #[tokio::test]
    async fn test_execute_returns_notes_with_kid_name() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let filter = NotesHistoryFilter {
            page_size: 10,
            ..Default::default()
        };
        let query = NotesHistoryQuery::new(filter);
        let result = query.execute_with_db(db_static).await.unwrap();

        assert!(!result.notes.is_empty());
        // All notes should have kid names
        for note in &result.notes {
            assert!(!note.kid_name.is_empty());
        }
    }

    #[tokio::test]
    async fn test_execute_filters_by_kid_id() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let filter = NotesHistoryFilter {
            kid_id: Some(1),
            page_size: 10,
            ..Default::default()
        };
        let query = NotesHistoryQuery::new(filter);
        let result = query.execute_with_db(db_static).await.unwrap();

        for note in &result.notes {
            assert_eq!(note.kid_id, 1);
        }
    }

    #[tokio::test]
    async fn test_execute_paginates_correctly() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let filter = NotesHistoryFilter {
            page_size: 5,
            ..Default::default()
        };
        let query = NotesHistoryQuery::new(filter);
        let result = query.execute_with_db(db_static).await.unwrap();

        assert!(result.notes.len() <= 5);
        assert!(result.total_pages >= 1);
    }

    #[tokio::test]
    async fn test_cursor_carries_filter_context() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        // First page with filter
        let filter = NotesHistoryFilter {
            kid_id: Some(1),
            page_size: 2,
            sort_by: Some("created_at".to_string()),
            sort_order: Some("desc".to_string()),
            ..Default::default()
        };
        let query = NotesHistoryQuery::new(filter);
        let result = query.execute_with_db(db_static).await.unwrap();

        if let Some(cursor) = result.cursor {
            // Use cursor with DIFFERENT filter params - they should be ignored
            let filter2 = NotesHistoryFilter {
                kid_id: Some(999), // This should be ignored
                page_size: 100,   // This should be ignored
                cursor: Some(cursor),
                ..Default::default()
            };
            let query2 = NotesHistoryQuery::new(filter2);
            let result2 = query2.execute_with_db(db_static).await.unwrap();

            // Should still get kid_id=1 results because cursor carries the context
            for note in &result2.notes {
                assert_eq!(note.kid_id, 1);
            }
        }
    }

    async fn setup_test_db() -> libsql::Connection {
        crate::backend::test_db::setup_test_db().await
    }
}
