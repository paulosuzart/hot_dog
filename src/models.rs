use chrono::DateTime;
use dioxus::fullstack::serde::Serialize;
use serde::Deserialize;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CountAggregation {
    Monthly(u8),
    Weekly(u8),
    Daily(u8),
    Yearly(u8),
}

impl CountAggregation {
    /// Returns the aggregation period label (e.g. "Monthly", "Weekly").
    pub fn label(&self) -> &'static str {
        match self {
            CountAggregation::Monthly(_) => "Monthly",
            CountAggregation::Weekly(_) => "Weekly",
            CountAggregation::Daily(_) => "Daily",
            CountAggregation::Yearly(_) => "Yearly",
        }
    }

    /// Returns the current unit value for this aggregation period.
    pub fn unit(&self) -> u8 {
        match self {
            CountAggregation::Monthly(v)
            | CountAggregation::Weekly(v)
            | CountAggregation::Daily(v)
            | CountAggregation::Yearly(v) => *v,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CountMetadata {
    pub aggregation: CountAggregation,
}

#[derive(Clone)]
pub enum KidsResponseWrapper {
    Loading,
    Loaded(GetKidsResponse),
    NoKids,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GetKidsResponse {
    pub kids: Vec<Kid>,
    pub count_metadata: CountMetadata,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Kid {
    pub name: String,
    pub id: u32,
    pub count: u8,
    pub latest_note: DateTime<chrono::Utc>,
}
