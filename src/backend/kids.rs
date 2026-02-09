use crate::models::{CountAggregation, GetKidsResponse};
#[cfg(feature = "server")]
use crate::models::{CountMetadata, Kid};

#[cfg(feature = "server")]
use crate::backend::turso::get_db;

use chrono::Datelike;
#[cfg(feature = "server")]
use libsql::de;

use dioxus::prelude::*;

#[server]
pub async fn decrement_kid_count(kid_id: u32) -> Result<(), ServerFnError> {
    // Here you would typically interact with your database to decrement the count for the specified kid.
    // For demonstration purposes, we'll just print the kid_id.
    println!("Decrementing count for kid with ID: {}", kid_id);
    Ok(())
}

#[server]
pub async fn increment_kid_count(kid_id: u32) -> Result<(), ServerFnError> {
    // Here you would typically interact with your database to increment the count for the specified kid.
    // For demonstration purposes, we'll just print the kid_id.
    println!("Incrementing count for kid with ID: {}", kid_id);
    Ok(())
}

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
#[server]
pub async fn get_granularity() -> Result<String, ServerFnError> {
    let settings = get_count_metadata().await?;
    Ok(settings.granularity)
}

#[cfg(feature = "server")]
const ALLOWED_GRANULARITIES: &[&str] = &["DAILY", "WEEKLY", "MONTHLY", "YEARLY"];

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
