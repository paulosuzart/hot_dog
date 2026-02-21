#[cfg(feature = "server")]
use chrono::NaiveDateTime;
#[cfg(feature = "server")]
use dioxus::server::ServerFnError;

#[cfg(feature = "server")]
use crate::backend::db::QueryExecutor;

#[cfg(feature = "server")]
use crate::backend::kids::Granularity;

#[cfg(feature = "server")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HistoryDetailResponse {
    pub kid_id: u32,
    pub notes: Vec<NoteDetails>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NoteDetails {
    pub kid_id: u32,
    pub quantity: i32,
    pub created_at: NaiveDateTime,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryDetailsQuery {
    pub kid_id: u32,
    pub period: String,
    pub granularity: Granularity,
}

#[cfg(feature = "server")]
impl HistoryDetailsQuery {
    pub fn new(kid_id: u32, period: String, granularity: Granularity) -> Self {
        Self {
            kid_id,
            period,
            granularity,
        }
    }
}

#[cfg(feature = "server")]
impl QueryExecutor for HistoryDetailsQuery {
    type Output = HistoryDetailResponse;
    type Error = ServerFnError;

    async fn execute_query(&self, conn: &libsql::Connection) -> Result<Self::Output, Self::Error> {
        let grain_format = self.granularity.grain_format();
        let grain_value = self
            .granularity
            .grain_value(&self.period)
            .map_err(|e| ServerFnError::new(format!("Failed to parse grain value: {}", e)))?;

        let query = format!(
            "
        SELECT
            notes.created_at,
            quantity
        FROM
            notes
        WHERE
            kid_id = :kid_id
            AND strftime('{grain_format}', notes.created_at) = :grain_value
        "
        );

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
        })
    }
}
