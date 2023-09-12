use std::{fmt, str::FromStr};

use async_graphql::{Enum, InputObject, SimpleObject};
pub use cube_client::models::{v1_time::TimeGranularity, V1LoadResponse};
use either::Either;
use hub_core::{
    anyhow::Result,
    chrono::{NaiveDate, NaiveDateTime},
    uuid::Uuid,
};
use serde::{de, Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, SimpleObject, Deserialize)]
pub struct MintDataPoint {
    #[serde(
        deserialize_with = "parse_count",
        skip_serializing_if = "Option::is_none",
        rename(deserialize = "mints.count"),
        default
    )]
    pub count: Option<u64>,
    #[serde(
        deserialize_with = "parse_timestamp",
        skip_serializing_if = "Option::is_none",
        rename(deserialize = "mints.timestamp"),
        default
    )]
    pub timestamp: Option<NaiveDateTime>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(deserialize = "mints.collection_id"),
        deserialize_with = "parse_uuid",
        default
    )]
    pub collection_id: Option<Uuid>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(deserialize = "mints.project_id"),
        deserialize_with = "parse_uuid",
        default
    )]
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Clone, SimpleObject, Deserialize)]
pub struct CustomerDataPoint {
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(deserialize = "customers.count"),
        deserialize_with = "parse_count",
        default
    )]
    pub count: Option<u64>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(deserialize = "customers.timestamp"),
        deserialize_with = "parse_timestamp",
        default
    )]
    pub timestamp: Option<NaiveDateTime>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(deserialize = "customers.project_id"),
        deserialize_with = "parse_uuid",
        default
    )]
    pub project_id: Option<Uuid>,
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

#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
pub enum Resource {
    Mints,
    Customers,
    Wallets,
    Collections,
    Projects,
    Transfers,
    Webhooks,
    Credits,
}

impl fmt::Display for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Resource::Mints => "mints",
            Resource::Customers => "customers",
            Resource::Wallets => "wallets",
            Resource::Collections => "collections",
            Resource::Projects => "projects",
            Resource::Transfers => "transfers",
            Resource::Webhooks => "webhooks",
            Resource::Credits => "credits",
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
            "transfers" => Ok(Resource::Transfers),
            "webhooks" => Ok(Resource::Webhooks),
            "credits" => Ok(Resource::Credits),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Enum, Copy, Clone, Eq, PartialEq)]
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

#[derive(InputObject)]
pub struct DateRange {
    pub start: Option<NaiveDate>,
    pub end: Option<NaiveDate>,
    pub interval: Option<Interval>,
}

#[derive(Debug, Default, Enum, Copy, Clone, Eq, PartialEq)]
pub enum Interval {
    All,
    #[default]
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

    #[must_use]
    pub fn to_date_range(&self) -> Either<String, Vec<String>> {
        match self {
            Self::All => Either::Right(vec![]),
            Self::ThisWeek
            | Self::Today
            | Self::Yesterday
            | Self::Last7Days
            | Self::LastWeek
            | Self::ThisMonth
            | Self::Last30Days
            | Self::LastMonth
            | Self::LastQuarter
            | Self::ThisYear
            | Self::LastYear => Either::Left(self.to_string()),
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

fn parse_count<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(val) => val.parse::<u64>().map(Some).map_err(de::Error::custom),
        None => Ok(None),
    }
}

fn parse_timestamp<'de, D>(deserializer: D) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(val) => NaiveDateTime::parse_from_str(&val, "%Y-%m-%dT%H:%M:%S%.f")
            .map(Some)
            .map_err(de::Error::custom),
        None => Ok(None),
    }
}

fn parse_uuid<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(val) => Uuid::parse_str(&val).map(Some).map_err(de::Error::custom),
        None => Ok(None),
    }
}
