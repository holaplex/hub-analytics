use std::{fmt, str::FromStr};

use async_graphql::{Enum, InputObject, SimpleObject};
pub use cube_client::models::{v1_time::TimeGranularity, V1LoadResponse};
use hub_core::{
    anyhow::Result,
    chrono::{NaiveDate, NaiveDateTime},
    uuid::Uuid,
};
use serde::Deserialize;
use serde_json::Value;

/// A `DataPoint` object containing analytics information.
#[derive(Debug, Clone, Deserialize, SimpleObject)]
pub struct DataPoint {
    /// Analytics data for mints.
    pub mints: Option<Data>,
    /// Analytics data for customers.
    pub customers: Option<Data>,
    /// Analytics data for collections.
    pub collections: Option<Data>,
    /// Analytics data for wallets.
    pub wallets: Option<Data>,
    /// Analytics data for projects.
    pub projects: Option<Data>,
}

#[derive(Debug, Clone, Deserialize, SimpleObject)]
pub struct Data {
    /// Count for the metric.
    pub count: u64,
    /// The ID of the organization the data belongs to.
    pub organization_id: Option<Uuid>,
    /// The ID of the collection the data belongs to.
    pub collection_id: Option<Uuid>,
    /// The ID of the project the data belongs to.
    pub project_id: Option<Uuid>,
    /// The timestamp associated with the data point.
    pub timestamp: Option<NaiveDateTime>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Granularity {
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl fmt::Display for Granularity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Granularity::Hour => "hour",
            Granularity::Day => "day",
            _ => "week",
        };
        write!(f, "{s}")
    }
}

#[derive(InputObject)]
pub struct Measure {
    pub resource: Resource,
    pub operation: Operation,
}

impl Measure {
    #[must_use]
    pub fn new(resource: Resource, operation: Operation) -> Self {
        Self {
            resource,
            operation,
        }
    }
    #[must_use]
    pub fn as_string(&self) -> String {
        format!("{}.{}", self.resource, self.operation)
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Operation {
    Count,
    Change,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Operation::Count => "count",
            Operation::Change => "change",
        };
        write!(f, "{s}")
    }
}
#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Resource {
    Mints,
    Customers,
    Wallets,
    Collections,
    Projects,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Resource::Mints => "mints",
            Resource::Customers => "customers",
            Resource::Wallets => "wallets",
            Resource::Collections => "collections",
            Resource::Projects => "projects",
        };
        write!(f, "{s}")
    }
}

impl FromStr for Resource {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mints" => Ok(Resource::Mints),
            "customers" => Ok(Resource::Customers),
            "wallets" => Ok(Resource::Wallets),
            "collections" => Ok(Resource::Collections),
            "projects" => Ok(Resource::Projects),
            _ => Err(()),
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Order {
    Asc,
    Desc,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Order::Asc => "asc",
            Order::Desc => "desc",
        };
        write!(f, "{s}")
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Dimension {
    Collections,
    Projects,
    Organizations,
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Dimension::Collections => "collections",
            Dimension::Projects => "projects",
            Dimension::Organizations => "organizations",
        };
        write!(f, "{s}")
    }
}

#[derive(InputObject)]
pub struct DateRange {
    pub start: Option<NaiveDate>,
    pub end: Option<NaiveDate>,
    pub interval: Option<Interval>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum Interval {
    All,
    Today,
    Yesterday,
    ThisWeek,
    ThisMonth,
    ThisYear,
    Last7Days,
    Last30Days,
    LastWeek,
    LastMonth,
    LastQuarter,
    LastYear,
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Interval::All => "all",
            Interval::Today => "today",
            Interval::Yesterday => "yesterday",
            Interval::ThisWeek => "this week",
            Interval::ThisMonth => "this month",
            Interval::ThisYear => "this year",
            Interval::Last7Days => "last 7 days",
            Interval::Last30Days => "last 30 days",
            Interval::LastWeek => "last week",
            Interval::LastMonth => "last month",
            Interval::LastQuarter => "last quarter",
            Interval::LastYear => "last year",
        };
        write!(f, "{s}")
    }
}
impl Interval {
    #[must_use]
    pub fn to_granularity(&self) -> Granularity {
        match self {
            Interval::Today | Interval::Yesterday => Granularity::Hour,
            Interval::ThisWeek
            | Interval::All
            | Interval::Last7Days
            | Interval::LastWeek
            | Interval::ThisMonth
            | Interval::Last30Days
            | Interval::LastMonth => Granularity::Day,
            Interval::LastQuarter => Granularity::Week,
            Interval::ThisYear | Interval::LastYear => Granularity::Month,
        }
    }
}
impl From<DateRange> for Vec<String> {
    fn from(date_range: DateRange) -> Self {
        vec![
            date_range.start.unwrap().format("%Y-%m-%d").to_string(),
            date_range.end.unwrap().format("%Y-%m-%d").to_string(),
        ]
    }
}

impl From<Granularity> for TimeGranularity {
    fn from(input: Granularity) -> Self {
        match input {
            Granularity::Hour => TimeGranularity::Minute,
            Granularity::Day => TimeGranularity::Hour,
            Granularity::Week | Granularity::Month => TimeGranularity::Day,
            Granularity::Year => TimeGranularity::Month,
        }
    }
}

pub struct DataPoints(Vec<DataPoint>);
impl DataPoints {
    #[must_use]
    pub fn into_vec(self) -> Vec<DataPoint> {
        self.0
    }
}
impl DataPoints {
    /// Helper function to get a field and parse it as u64.
    fn parse_count(value: &Value, resource: &str) -> Option<u64> {
        value
            .get(&format!("{resource}.count"))
            .and_then(Value::as_str)
            .and_then(|s| s.parse().ok())
    }

    /// Helper function to get a field and parse it as Uuid.
    fn parse_uuid(value: &Value, field: &str) -> Option<Uuid> {
        value
            .get(field)
            .and_then(Value::as_str)
            .and_then(|s| Uuid::parse_str(s).ok())
    }

    /// Helper function to get a field and parse it as `NaiveDateTime`.
    fn parse_datetime(value: &Value, field: &str) -> Option<NaiveDateTime> {
        value
            .get(field)
            .and_then(Value::as_str)
            .and_then(|s| NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f").ok())
    }

    /// # Returns
    /// a vector of datapoints parsed from the response coming from Cube API
    ///
    /// # Errors
    /// This function returns an error if there was a problem with retrieving the data points.
    pub fn from_response(
        response: &str,
        resource: &str,
    ) -> Result<Vec<DataPoint>, async_graphql::Error> {
        let response: V1LoadResponse =
            serde_json::from_str(response).map_err(|e| async_graphql::Error::new(e.to_string()))?;

        hub_core::tracing::info!("Res: {:#?}", response);
        let data: Vec<Value> = response
            .results
            .first()
            .ok_or_else(|| async_graphql::Error::new("No results found"))?
            .data
            .clone();

        data.into_iter()
            .map(|v| {
                Ok(DataPoint {
                    mints: Self::parse_data(&v, resource),
                    customers: Self::parse_data(&v, resource),
                    wallets: Self::parse_data(&v, resource),
                    collections: Self::parse_data(&v, resource),
                    projects: Self::parse_data(&v, resource),
                })
            })
            .collect()
    }

    fn parse_data(value: &Value, resource: &str) -> Option<Data> {
        Some(Data {
            count: Self::parse_count(value, resource)?,
            organization_id: Self::parse_uuid(value, "projects.organization_id"),
            project_id: Self::parse_uuid(value, &format!("{resource}.project_id")),
            collection_id: Self::parse_uuid(value, "mints.collection_id"),
            timestamp: Self::parse_datetime(value, &format!("{resource}.timestamp")),
        })
    }
}
