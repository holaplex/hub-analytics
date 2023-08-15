use std::fmt;

use async_graphql::{Enum, InputObject, SimpleObject};
pub use cube_client::models::{v1_time::TimeGranularity, V1LoadResponse};
use hub_core::{
    anyhow::Result,
    chrono::{NaiveDate, NaiveDateTime},
    uuid::Uuid,
};
use serde::Deserialize;
use serde_aux::prelude::*;

/// A `DataPoint` object containing analytics information.
#[derive(Debug, Clone, Deserialize, SimpleObject)]
pub struct DataPoint {
    /// Count of the metric.
    #[serde(
        rename = "mints.count",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub count: u64,
    /// The ID of the organization the data belongs to.
    pub organization_id: Option<Uuid>,
    /// The ID of the collection the data belongs to.
    #[serde(rename = "mints.collection_id")]
    pub collection_id: Option<Uuid>,
    /// The ID of the project the data belongs to.
    #[serde(rename = "mints.project_id")]
    pub project_id: Option<Uuid>,
    /// The timestamp associated with the data point.
    #[serde(rename = "mints.timestamp")]
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

#[derive(InputObject)]
pub struct Measure {
    pub resource: Resource,
    pub operation: Operation,
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
    Organizations,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Resource::Mints => "mints",
            Resource::Customers => "customers",
            Resource::Wallets => "wallets",
            Resource::Collections => "collections",
            Resource::Projects => "projects",
            Resource::Organizations => "organizations",
        };
        write!(f, "{s}")
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
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl From<DateRange> for Vec<String> {
    fn from(date_range: DateRange) -> Self {
        vec![
            date_range.start_date.format("%Y-%m-%d").to_string(),
            date_range.end_date.format("%Y-%m-%d").to_string(),
        ]
    }
}

impl From<Granularity> for TimeGranularity {
    fn from(input: Granularity) -> Self {
        match input {
            Granularity::Hour => TimeGranularity::Hour,
            Granularity::Day => TimeGranularity::Day,
            Granularity::Week => TimeGranularity::Week,
            Granularity::Month => TimeGranularity::Month,
            Granularity::Year => TimeGranularity::Year,
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
impl TryFrom<String> for DataPoints {
    type Error = hub_core::anyhow::Error;

    fn try_from(response: String) -> Result<Self> {
        let response_data: V1LoadResponse = serde_json::from_str(&response)
            .map_err(|e| async_graphql::Error::new(e.to_string()))
            .unwrap();

        let data: Vec<serde_json::Value> = response_data
            .results
            .first()
            .ok_or_else(|| async_graphql::Error::new("No results found"))
            .unwrap()
            .data
            .clone();

        let data_points: Vec<DataPoint> = data
            .into_iter()
            .map(serde_json::from_value)
            .collect::<Result<_, _>>()
            .map_err(|e| async_graphql::Error::new(e.to_string()))
            .unwrap();

        Ok(DataPoints(data_points))
    }
}
