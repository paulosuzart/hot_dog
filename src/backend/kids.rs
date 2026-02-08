use crate::models::GetKidsResponse;
#[cfg(feature = "server")]
use crate::models::{CountAggregation, CountMetadata, Kid};
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

#[server]
pub async fn get_kids() -> Result<GetKidsResponse, ServerFnError> {
    let kids = vec![
        Kid {
            name: "Paulo".to_string(),
            id: 0,
            count: 3,
            latest_note: chrono::Utc::now(),
        },
        Kid {
            name: "Maria".to_string(),
            id: 1,
            count: 5,
            latest_note: chrono::Utc::now(),
        },
    ];

    let response = GetKidsResponse {
        kids,
        count_metadata: CountMetadata {
            aggregation: CountAggregation::Monthly(10),
        },
    };
    Ok(response)
}
