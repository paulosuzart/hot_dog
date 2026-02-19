use crate::components::kid_history::KidHistoryPage;
use crate::models::{
    CountAggregation, GetKidsResponse, KidHistory, KidHistoryResponse, KidSummary,
};

#[cfg(feature = "server")]
use std::fmt::Display;

#[cfg(feature = "server")]
use std::str::FromStr;

#[cfg(feature = "server")]
use base64::{engine::general_purpose::URL_SAFE, Engine as _};

#[cfg(feature = "server")]
use crate::models::{CountMetadata, Kid};
#[cfg(feature = "server")]
use std::collections::HashMap;
#[cfg(feature = "server")]
use std::fmt;
#[cfg(feature = "server")]
use std::sync::{LazyLock, Mutex};

#[cfg(feature = "server")]
use crate::backend::turso::get_db;

#[cfg(feature = "server")]
use chrono::Datelike;

#[cfg(feature = "server")]
use libsql::de;

use dioxus::prelude::*;

#[cfg(feature = "server")]
const ALLOWED_GRANULARITIES: &[&str] = &["DAILY", "WEEKLY", "MONTHLY", "YEARLY"];

#[cfg(feature = "server")]
const MAX_KID_NAME_LENGTH: usize = 17;

#[cfg(feature = "server")]
const FORMAT_MAP: LazyLock<HashMap<&str, &str>> = LazyLock::new(|| {
    HashMap::from([
        ("DAILY", "%Y-%m-%d"),
        ("WEEKLY", "%Y-W%W"),
        ("MONTHLY", "%Y-%m"),
        ("YEARLY", "%Y"),
    ])
});

#[server]
pub async fn decrement_kid_count(kid_id: u32) -> Result<(), ServerFnError> {
    // Here you would typically interact with your database to decrement the count for the specified kid.
    // For demonstration purposes, we'll just print the kid_id.
    log_note(kid_id, false).await
}

#[server]
pub async fn increment_kid_count(kid_id: u32) -> Result<(), ServerFnError> {
    // Here you would typically interact with your database to increment the count for the specified kid.
    // For demonstration purposes, we'll just print the kid_id.
    log_note(kid_id, true).await
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct KidTotalRow {
    period: String,
    kid_id: u32,
    total: i32,
}

#[cfg(feature = "server")]
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct KidRow {
    id: u32,
    name: String,
    created_at: String,
}

#[derive(Debug, serde::Deserialize)]
#[cfg(feature = "server")]
struct SettingsRow {
    id: u32,
    granularity: String,
    created_at: String,
}

#[derive(Debug, serde::Deserialize)]
#[cfg(feature = "server")]
struct SummaryRow {
    period: Option<String>,
    total: Option<i32>,
    kid_id: u32,
    name: String,
    created_at: String,
    latest_note: Option<String>,
}

#[cfg(feature = "server")]
impl SummaryRow {
    fn to_kid(&self) -> Kid {
        Kid {
            id: self.kid_id,
            name: self.name.clone(),
            count: self.total.unwrap_or(0) as i8,
            latest_note: self
                .latest_note
                .as_deref()
                .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").ok()),
        }
    }
}

#[cfg(feature = "server")]
#[derive(Debug, serde::Deserialize)]
struct HistoryRow {
    kid_id: u32,
    period: String,
    total: i32,
    result: i32,
    neg_count: i32,
    post_count: i32,
    name: String,
}

#[cfg(feature = "server")]
impl Into<KidHistory> for HistoryRow {
    fn into(self) -> KidHistory {
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

/// Get the current cycle based on the settings granularity.
#[cfg(feature = "server")]
fn get_current_cycle(settings: &SettingsRow) -> CountAggregation {
    let now = chrono::offset::Utc::now().naive_utc();

    match settings.granularity.as_str() {
        "DAILY" => CountAggregation::Daily(now.day(), now.month(), now.year() as u32),
        "WEEKLY" => CountAggregation::Weekly(now.iso_week().week(), now.month(), now.year() as u32),
        "MONTHLY" => CountAggregation::Monthly(now.month(), now.year() as u32),
        _ => CountAggregation::Monthly(now.month(), now.year() as u32), // default to monthly if unrecognized
    }
}

#[cfg(feature = "server")]
async fn get_count_metadata() -> Result<SettingsRow, ServerFnError> {
    let conn = get_db().await;

    let mut rows = conn
        .query(
            "SELECT id, granularity, created_at FROM settings LIMIT 1",
            (),
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        let settings_row =
            de::from_row::<SettingsRow>(&row).map_err(|e| ServerFnError::new(e.to_string()))?;

        Ok(settings_row)
    } else {
        Err(ServerFnError::new("No settings found".to_string()))
    }
}

/// Fetches the current granularity setting as a string (DAILY, WEEKLY, MONTHLY, YEARLY).
#[cfg(feature = "server")]
pub async fn log_note(kid_id: u32, add: bool) -> Result<(), ServerFnError> {
    let conn = get_db().await;
    let quantity = if add { 1 } else { -1 };
    let created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO notes (kid_id, quantity, created_at) VALUES (?1, ?2, ?3)",
        libsql::params![kid_id, quantity, created_at],
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

/// Fetches the current granularity setting as a string (DAILY, WEEKLY, MONTHLY, YEARLY).
#[server]
pub async fn get_granularity() -> Result<String, ServerFnError> {
    let settings = get_count_metadata().await?;
    Ok(settings.granularity)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Granularity {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl Granularity {
    fn grain_format(&self) -> &'static str {
        match self {
            Granularity::Daily => "%Y-%m-%d",
            Granularity::Weekly => "%Y-W%W",
            Granularity::Monthly => "%Y-%m",
            Granularity::Yearly => "%Y",
        }
    }
}

#[cfg(feature = "server")]
impl std::fmt::Display for Granularity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Granularity::Daily => "DAILY",
            Granularity::Weekly => "WEEKLY",
            Granularity::Monthly => "MONTHLY",
            Granularity::Yearly => "YEARLY",
        };
        write!(f, "{}", s)
    }
}

impl std::str::FromStr for Granularity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DAILY" => Ok(Self::Daily),
            "WEEKLY" => Ok(Self::Weekly),
            "MONTHLY" => Ok(Self::Monthly),
            "YEARLY" => Ok(Self::Yearly),
            _ => Err(format!("Invalid granularity: {}", s)),
        }
    }
}

/// Updates the granularity setting in the database.
/// Accepts: "DAILY", "WEEKLY", "MONTHLY", "YEARLY".
#[server]
pub async fn update_granularity(granularity: String) -> Result<(), ServerFnError> {
    if !ALLOWED_GRANULARITIES.contains(&granularity.as_str()) {
        return Err(ServerFnError::new(format!(
            "Invalid granularity: '{granularity}'. Must be one of: {ALLOWED_GRANULARITIES:?}"
        )));
    }
    let conn = get_db().await;
    conn.execute(
        "UPDATE settings SET granularity = ?1 WHERE id = 1",
        libsql::params![granularity],
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

/// Helper function to determine the grain format and value for SQL queries based on the current granularity setting.
/// TODO: Move to granularity itself
// #[cfg(feature = "server")]
fn get_grain(
    granularity: Granularity,
    now: &chrono::NaiveDateTime,
) -> Result<(&'static str, String), ServerFnError> {
    use chrono::Datelike;
    return match granularity {
        Granularity::Daily => Ok(("%Y-%m-%d", now.format("%Y-%m-%d").to_string())),
        Granularity::Weekly => {
            let week_start =
                *now - chrono::Duration::days(now.weekday().num_days_from_monday() as i64);
            Ok(("%Y-W%W", week_start.format("%Y-%m-%d 00:00:00").to_string()))
        }
        Granularity::Monthly => Ok(("%Y-%m", now.format("%Y-%m").to_string())),
        Granularity::Yearly => Ok(("%Y", now.format("%Y").to_string())),
        _ => {
            return Err(ServerFnError::new(
                "Invalid granularity in settings".to_string(),
            ))
        }
    };
}

/// Fetches the list of kids along with their count metadata.
/// Intended to be used at the home screen
#[server]
pub async fn get_kids() -> Result<GetKidsResponse, ServerFnError> {
    let conn = get_db().await;
    let now = chrono::offset::Utc::now().naive_utc();
    let meta_raw = get_count_metadata().await?;

    let (grain_format, grain_value) = match meta_raw.granularity.as_str() {
        "DAILY" => ("%Y-%m-%d", now.format("%Y-%m-%d").to_string()),
        "WEEKLY" => {
            let week_start =
                now - chrono::Duration::days(now.weekday().num_days_from_monday() as i64);
            ("%Y-W%W", week_start.format("%Y-%m-%d 00:00:00").to_string())
        }
        "MONTHLY" => ("%Y-%m", now.format("%Y-%m").to_string()),
        "YEARLY" => ("%Y", now.format("%Y").to_string()),
        _ => {
            return Err(ServerFnError::new(
                "Invalid granularity in settings".to_string(),
            ))
        }
    };

    let query = format!(
        "
    SELECT
        strftime('{}', notes.created_at) AS period,
        SUM(quantity) AS total,
        kids.id AS kid_id,
        kids.name as name,
        kids.created_at AS created_at,
        MAX(notes.created_at) AS latest_note
    FROM kids
    LEFT JOIN notes ON notes.kid_id = kids.id AND notes.created_at >= :grain_value
    GROUP BY kids.id, period",
        grain_format
    );

    tracing::debug!("SQL get_kids: {}, grain_value: {}", query, grain_value);
    let stm = conn
        .prepare(&query)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut rows = stm
        .query(libsql::named_params! { ":grain_value": grain_value })
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut kids = Vec::new();

    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        let kid_row =
            de::from_row::<SummaryRow>(&row).map_err(|e| ServerFnError::new(e.to_string()))?;
        kids.push(kid_row.to_kid());
    }

    let aggregation = get_current_cycle(&meta_raw);

    let response = GetKidsResponse {
        kids,
        count_metadata: CountMetadata {
            aggregation: aggregation,
        },
    };
    Ok(response)
}

/// Fetches just the list of kids (id + name) without count metadata.
/// Intended for the settings/management screen.
#[server]
pub async fn list_kids() -> Result<Vec<KidSummary>, ServerFnError> {
    let conn = get_db().await;
    let mut rows = conn
        .query("SELECT id, name FROM kids ORDER BY name ASC", ())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut kids = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        let kid =
            de::from_row::<KidSummary>(&row).map_err(|e| ServerFnError::new(e.to_string()))?;
        kids.push(kid);
    }
    Ok(kids)
}

/// Adds a new kid. Enforces a maximum of 10 kids.
#[server]
pub async fn add_kid(name: String) -> Result<KidSummary, ServerFnError> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(ServerFnError::new("Name cannot be empty".to_string()));
    }
    if name.len() > MAX_KID_NAME_LENGTH {
        return Err(ServerFnError::new(
            "Name too long (max 50 characters)".to_string(),
        ));
    }

    let conn = get_db().await;

    let mut trx = conn
        .transaction_with_behavior(libsql::TransactionBehavior::Deferred)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Enforce 10-kid limit
    let mut count_rows = trx
        .query("SELECT COUNT(*) as cnt FROM kids", ())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if let Some(row) = count_rows
        .next()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        let count: u32 = row.get(0).map_err(|e| ServerFnError::new(e.to_string()))?;
        if count >= 10 {
            return Err(ServerFnError::new("Maximum of 10 kids allowed".to_string()));
        }
    }

    let created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    trx.execute(
        "INSERT INTO kids (name, created_at) VALUES (?1, ?2)",
        libsql::params![name.clone(), created_at],
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let inserted_id = trx.last_insert_rowid();

    tracing::debug!("Inserted kid with ID: {}", inserted_id);
    trx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(KidSummary {
        id: inserted_id as u32,
        name,
    })
}

/// Deletes a kid by id.
#[server]
pub async fn delete_kid(kid_id: u32) -> Result<(), ServerFnError> {
    let conn = get_db().await;
    conn.execute("DELETE FROM kids WHERE id = ?1", libsql::params![kid_id])
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

/// Renames a kid.
#[server]
pub async fn rename_kid(kid_id: u32, new_name: String) -> Result<(), ServerFnError> {
    let new_name = new_name.trim().to_string();
    if new_name.is_empty() {
        return Err(ServerFnError::new("Name cannot be empty".to_string()));
    }
    if new_name.len() > MAX_KID_NAME_LENGTH {
        return Err(ServerFnError::new(
            "Name too long (max 50 characters)".to_string(),
        ));
    }

    let conn = get_db().await;
    conn.execute(
        "UPDATE kids SET name = ?1 WHERE id = ?2",
        libsql::params![new_name, kid_id],
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[cfg(feature = "server")]
#[derive(Debug, PartialEq)]
struct Cursor {
    grain_value: String,
    grain_format: &'static str,
    granularity: Granularity,
}

#[cfg(feature = "server")]
impl std::str::FromStr for Cursor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use chrono::Datelike;
        let decoded_cursor = URL_SAFE
            .decode(s)
            .map_err(|e| format!("Failed to decode cursor: {}. Error: {}", s, e))?;

        let str_cursor = String::from_utf8(decoded_cursor)
            .map_err(|e| format!("Invalid UTF-8 in cursor: {}. Error: {}", s, e))?;

        let (gt, granularity) = str_cursor.split_once('|').ok_or_else(|| {
            format!(
                "Invalid cursor format: {}. Expected 'timestamp|granularity'",
                s
            )
        })?;

        let cursor_granularity = Granularity::from_str(&granularity)
            .map_err(|e| format!("Invalid granularity in cursor: {}. Error: {}", s, e))?;

        let grain_value = match cursor_granularity {
            Granularity::Daily => {
                chrono::NaiveDate::parse_from_str(&gt, "%Y-%m-%d")
                    .map_err(|e| format!("Invalid date in cursor: {}. Error: {}", gt, e))?;
                gt.to_string()
            }
            Granularity::Weekly => {
                chrono::NaiveDate::parse_from_str(&gt, "%Y-%m-%d")
                    .map_err(|e| format!("Invalid date in cursor: {}. Error: {}", gt, e))?;
                gt.to_string()
            }
            Granularity::Monthly => {
                chrono::NaiveDate::parse_from_str(&gt, "%Y-%m")
                    .map_err(|e| format!("Invalid date in cursor: {}. Error: {}", gt, e))?;
                gt.to_string()
            }
            Granularity::Yearly => {
                chrono::NaiveDate::parse_from_str(&gt, "%Y")
                    .map_err(|e| format!("Invalid date in cursor: {}. Error: {}", gt, e))?;
                gt.to_string()
            }
        };

        Ok(Cursor {
            grain_value,
            grain_format: cursor_granularity.grain_format(),
            granularity: cursor_granularity,
        })
    }
}

#[server]
pub async fn get_paged_history(
    kid_id: u32,
    cursor: Option<String>,
    page_size: u8,
) -> Result<KidHistoryResponse, ServerFnError> {
    let conn = get_db().await;
    let metadata_raw = get_count_metadata().await?;
    let metadata_granualrity = Granularity::from_str(&metadata_raw.granularity)
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let curos_clause = if let Some(cursor_str) = cursor {
        let parsed_cursor = Cursor::from_str(&cursor_str).map_err(|e| ServerFnError::new(e))?;
        if parsed_cursor.granularity
            != metadata_raw
                .granularity
                .parse::<Granularity>()
                .map_err(|e| ServerFnError::new(e.to_string()))?
        {
            return Err(ServerFnError::new(format!(
                "Cursor granularity '{}' does not match current settings granularity '{}'",
                parsed_cursor.granularity, metadata_raw.granularity
            )));
        }

        format!(
            "AND strftime('{}', notes.created_at) < '{}'",
            metadata_granualrity.grain_format(),
            parsed_cursor.grain_value
        )
    } else {
        "".to_string()
    };

    let limit = page_size + 1; // Fetch one extra to determine if there's a next page

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
        metadata_granualrity.grain_format(),
        curos_clause
    );

    tracing::debug!("SQL get_paged_history: {}", query);

    let stm = conn
        .prepare(&query)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut rows = stm
        // TODO add grain value as named param
        .query(libsql::named_params! { ":limit": limit, ":kid_id": kid_id })
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut kid_history: Vec<KidHistory> = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        let kid =
            de::from_row::<HistoryRow>(&row).map_err(|e| ServerFnError::new(e.to_string()))?;
        kid_history.push(kid.into());
    }

    let next_cursor = if kid_history.len() as u8 > page_size {
        let last_item = kid_history.pop().unwrap();
        let cursor_str = format!("{}|{}", last_item.period, metadata_raw.granularity);
        Some(URL_SAFE.encode(cursor_str.as_bytes()))
    } else {
        None
    };

    Ok(KidHistoryResponse {
        history: kid_history,
        cursor: next_cursor,
        granularity: metadata_raw.granularity,
    })
}
