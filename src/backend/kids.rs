use crate::models::GetKidsResponse;
#[cfg(feature = "server")]
use crate::models::{CountAggregation, CountMetadata, Kid};

#[cfg(feature = "server")]
use crate::backend::turso::get_db;

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
struct KidRow {
    id: u32,
    name: String,
    created_at: String,
}

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

    let response = GetKidsResponse {
        kids,
        count_metadata: CountMetadata {
            aggregation: CountAggregation::Monthly(10),
        },
    };
    Ok(response)
}
