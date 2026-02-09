use chrono::NaiveDateTime;
use dioxus::fullstack::serde::Serialize;
use serde::Deserialize;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CountAggregation {
    /// month, year
    Monthly(u32, u32),
    /// week, month, year
    Weekly(u32, u32, u32),
    /// day, month, year
    Daily(u32, u32, u32),
    Yearly(u32),
}

impl CountAggregation {
    /// Returns the aggregation period label (e.g. "Monthly", "Weekly").
    pub fn label(&self) -> &'static str {
        match self {
            CountAggregation::Monthly(_, _) => "Monthly",
            CountAggregation::Weekly(_, _, _) => "Weekly",
            CountAggregation::Daily(_, _, _) => "Daily",
            CountAggregation::Yearly(_) => "Yearly",
        }
    }

    /// Returns the current unit value for this aggregation period.
    pub fn unit_str(&self) -> String {
        match self {
            CountAggregation::Monthly(m, y) => format!("{} of {}", m, y),
            CountAggregation::Weekly(w, m, y) => format!("{} of {} {}", w, m, y),
            CountAggregation::Daily(d, m, y) => format!("{} of {} {}", d, m, y),
            CountAggregation::Yearly(y) => format!("{}", y),
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
    pub latest_note: NaiveDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KidSummary {
    pub id: u32,
    pub name: String,
}
