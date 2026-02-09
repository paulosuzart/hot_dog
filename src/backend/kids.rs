use crate::models::{CountAggregation, GetKidsResponse, KidSummary};
#[cfg(feature = "server")]
use crate::models::{CountMetadata, Kid};
#[cfg(feature = "server")]
use std::collections::HashMap;
#[cfg(feature = "server")]
use std::sync::{LazyLock, Mutex};

#[cfg(feature = "server")]
use crate::backend::turso::get_db;

use chrono::Datelike;
#[cfg(feature = "server")]
use libsql::de;

use dioxus::prelude::*;

#[cfg(feature = "server")]
const ALLOWED_GRANULARITIES: &[&str] = &["DAILY", "WEEKLY", "MONTHLY", "YEARLY"];

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
#[allow(dead_code)]
struct SettingsRow {
    id: u32,
    granularity: String,
    created_at: String,
}

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

    conn.execute(
        "INSERT INTO notes (kid_id, quantity) VALUES (?1, ?2)",
        libsql::params![kid_id, quantity],
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

/// Fetches the list of kids along with their count metadata.
/// Intended to be used at the home screen
#[server]
pub async fn get_kids() -> Result<GetKidsResponse, ServerFnError> {
    let conn = get_db().await;

    let mut rows = conn
        .query("SELECT id, name, created_at FROM kids", ())
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut kids = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        let count = 5 as u8;
        let kid_row =
            de::from_row::<KidRow>(&row).map_err(|e| ServerFnError::new(e.to_string()))?;
        kids.push(Kid {
            id: kid_row.id,
            name: kid_row.name,
            count,
            // temporary. This is not the latest note.
            latest_note: chrono::NaiveDateTime::parse_from_str(
                &kid_row.created_at,
                "%Y-%m-%d %H:%M:%S",
            )
            .map_err(|e| ServerFnError::new(e.to_string()))?,
        });
    }

    // second query to complete metadata
    let meta_raw = get_count_metadata().await?;

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
    if name.len() > 50 {
        return Err(ServerFnError::new(
            "Name too long (max 50 characters)".to_string(),
        ));
    }

    let conn = get_db().await;

    // Enforce 10-kid limit
    let mut count_rows = conn
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

    conn.execute(
        "INSERT INTO kids (name, created_at) VALUES (?1, datetime('now', 'utc'))",
        libsql::params![name.clone()],
    )
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Retrieve the inserted kid
    let mut rows = conn
        .query(
            "SELECT id, name FROM kids WHERE name = ?1 ORDER BY id DESC LIMIT 1",
            libsql::params![name],
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    if let Some(row) = rows
        .next()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        let kid =
            de::from_row::<KidSummary>(&row).map_err(|e| ServerFnError::new(e.to_string()))?;
        Ok(kid)
    } else {
        Err(ServerFnError::new(
            "Failed to retrieve inserted kid".to_string(),
        ))
    }
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
    if new_name.len() > 50 {
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
