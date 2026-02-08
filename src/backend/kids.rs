use crate::notica_component::{CountAggregation, CountMetadata, GetKidsResponse, Kid};
use dioxus::prelude::*;

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
