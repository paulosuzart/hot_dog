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

#[cfg(feature = "server")]
pub struct NotesHistoryQuery {
    kid_id: Option<u32>,
    date_from: Option<String>,
    date_to: Option<String>,
    cursor: Option<String>,
    page_size: u8,
    sort_by: String,
    sort_order: String,
}

#[cfg(feature = "server")]
impl NotesHistoryQuery {
    pub fn new(
        kid_id: Option<u32>,
        date_from: Option<String>,
        date_to: Option<String>,
        cursor: Option<String>,
        page_size: u8,
        sort_by: Option<String>,
        sort_order: Option<String>,
    ) -> Self {
        let final_page_size = std::cmp::min(page_size, *MAX_NOTES_HISTORY_PAGE_SIZE);
        if final_page_size != page_size {
            tracing::warn!(
                "Using default page size. Request {} exceeded system max {}",
                page_size,
                *MAX_NOTES_HISTORY_PAGE_SIZE
            );
        }

        let sort_by = sort_by.unwrap_or_else(|| "created_at".to_string());
        let sort_order = sort_order.unwrap_or_else(|| "desc".to_string());

        Self {
            kid_id,
            date_from,
            date_to,
            cursor,
            page_size: final_page_size,
            sort_by,
            sort_order,
        }
    }

    fn get_sort_clause(&self) -> String {
        let sort_column = match self.sort_by.as_str() {
            "kid_name" => "kids.name",
            "created_at" | _ => "notes.created_at",
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
            // Cursor encodes the row_num of the last item in the current page
            let last_in_page = notes.last().unwrap();
            let cursor_str = format!(
                "{}|{}|{}",
                last_in_page.row_num, self.sort_by, self.sort_order
            );
            Some(URL_SAFE.encode(cursor_str.as_bytes()))
        } else {
            None
        }
    }

    /// Fetches notes with filtering, sorting, and cursor-based pagination.
    async fn fetch_notes(
        &self,
        conn: &'static libsql::Connection,
    ) -> Result<(Vec<NumberedNoteRow>, u32), ServerFnError> {
        let sort_clause = self.get_sort_clause();

        let query = format!(
            r#"
            WITH all_notes AS (
                SELECT
                    notes.id,
                    notes.kid_id,
                    kids.name as kid_name,
                    notes.quantity,
                    notes.created_at,
                    ROW_NUMBER() OVER ({sort_clause}) as row_num
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
            WHERE n.row_num > COALESCE(:cursor_row_num, 0)
            {sort_clause}
            LIMIT :limit
            "#
        );

        tracing::debug!("SQL fetch_notes: {}", query);

        let cursor_row_num = match &self.cursor {
            Some(c) => {
                let decoded = URL_SAFE
                    .decode(c)
                    .map_err(|e| ServerFnError::new(format!("Failed to decode cursor: {}", e)))?;
                let cursor_str = String::from_utf8(decoded)
                    .map_err(|e| ServerFnError::new(format!("Invalid UTF-8 in cursor: {}", e)))?;
                let parts: Vec<&str> = cursor_str.split('|').collect();
                if parts.len() >= 1 {
                    parts[0]
                        .parse::<u32>()
                        .map_err(|e| ServerFnError::new(format!("Invalid cursor row_num: {}", e)))?
                } else {
                    0
                }
            }
            None => 0,
        };

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
                ":cursor_row_num": cursor_row_num,
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
    use crate::backend::test_db::setup_test_db;

    #[test]
    fn test_new_creates_query_with_defaults() {
        let query = NotesHistoryQuery::new(None, None, None, None, 10, None, None);

        assert!(query.kid_id.is_none());
        assert!(query.date_from.is_none());
        assert!(query.date_to.is_none());
        assert!(query.cursor.is_none());
        assert_eq!(query.page_size, 10);
        assert_eq!(query.sort_by, "created_at");
        assert_eq!(query.sort_order, "desc");
    }

    #[test]
    fn test_new_with_all_params() {
        let query = NotesHistoryQuery::new(
            Some(1),
            Some("2026-01-01".to_string()),
            Some("2026-12-31".to_string()),
            None,
            20,
            Some("kid_name".to_string()),
            Some("asc".to_string()),
        );

        assert_eq!(query.kid_id, Some(1));
        assert_eq!(query.date_from, Some("2026-01-01".to_string()));
        assert_eq!(query.date_to, Some("2026-12-31".to_string()));
        assert_eq!(query.page_size, 20);
        assert_eq!(query.sort_by, "kid_name");
        assert_eq!(query.sort_order, "asc");
    }

    #[test]
    fn test_get_sort_clause_created_at_desc() {
        let query = NotesHistoryQuery::new(None, None, None, None, 10, None, None);
        let clause = query.get_sort_clause();

        assert!(clause.contains("notes.created_at"));
        assert!(clause.contains("DESC"));
    }

    #[test]
    fn test_get_sort_clause_kid_name_asc() {
        let query =
            NotesHistoryQuery::new(None, None, None, None, 10, Some("kid_name".to_string()), Some("asc".to_string()));
        let clause = query.get_sort_clause();

        assert!(clause.contains("kids.name"));
        assert!(clause.contains("ASC"));
    }

    #[tokio::test]
    async fn test_execute_returns_empty_for_no_data() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let query = NotesHistoryQuery::new(Some(999), None, None, None, 10, None, None);
        let result = query.execute_with_db(db_static).await.unwrap();

        assert_eq!(result.notes.len(), 0);
        assert_eq!(result.total_count, 0);
    }

    #[tokio::test]
    async fn test_execute_returns_notes_with_kid_name() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let query = NotesHistoryQuery::new(None, None, None, None, 10, None, None);
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

        let query = NotesHistoryQuery::new(Some(1), None, None, None, 10, None, None);
        let result = query.execute_with_db(db_static).await.unwrap();

        for note in &result.notes {
            assert_eq!(note.kid_id, 1);
        }
    }

    #[tokio::test]
    async fn test_execute_paginates_correctly() {
        let db = setup_test_db().await;
        let db_static = Box::leak(Box::new(db));

        let query = NotesHistoryQuery::new(None, None, None, None, 5, None, None);
        let result = query.execute_with_db(db_static).await.unwrap();

        assert!(result.notes.len() <= 5);
        assert!(result.total_pages >= 1);
    }
}
