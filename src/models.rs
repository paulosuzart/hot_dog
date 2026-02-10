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

    /// Returns a contextual label for the footer (e.g. "Current month", "Today").
    pub fn unit_label(&self) -> &'static str {
        match self {
            CountAggregation::Monthly(_, _) => "Current month",
            CountAggregation::Weekly(_, _, _) => "Current week",
            CountAggregation::Daily(_, _, _) => "Today",
            CountAggregation::Yearly(_) => "Current year",
        }
    }

    /// Returns the current unit value for this aggregation period.
    pub fn unit_str(&self) -> String {
        match self {
            CountAggregation::Monthly(m, y) => format!("{} {}", month_abbr(*m), y),
            CountAggregation::Weekly(w, m, y) => format!("W{} · {} {}", w, month_abbr(*m), y),
            CountAggregation::Daily(d, m, y) => format!("{} {} {}", d, month_abbr(*m), y),
            CountAggregation::Yearly(y) => format!("{}", y),
        }
    }
}

fn month_abbr(m: u32) -> &'static str {
    match m {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
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
    pub count: i8,
    pub latest_note: Option<NaiveDateTime>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KidSummary {
    pub id: u32,
    pub name: String,
}
