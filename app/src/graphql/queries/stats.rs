use std::fmt;

use async_graphql::{Context, Enum, InputObject, Object, Result};
use cube_client::models::{
    V1LoadRequestQueryFilterItem, V1LoadRequestQueryTimeDimension, V1LoadResponse,
};
use hub_core::{chrono::NaiveDate, uuid::Uuid};

use crate::{
    cube_client::{Client, Query as CubeQuery, TimeGranularity},
    graphql::objects::DataPoint,
};

#[derive(Debug, Clone, Default)]
pub struct Query;

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
    resource: Resource,
    operation: Operation,
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

#[Object(name = "StatQuery")]
impl Query {
    /// Returns a list of data points for a specific collection and timeframe.
    ///
    /// # Arguments
    /// * `collection_id` - The ID of the collection.
    /// * `granularity` - The time granularity for grouping (e.g., Day, Week, Month, Year).
    /// * `limit` - Optional limit on the number of data points to retrieve.
    ///
    /// # Returns
    /// A vector of Stats objects representing the analytics data.
    ///
    /// # Errors
    /// This function returns an error if there was a problem with retrieving the data points.
    #[allow(clippy::too_many_arguments)]
    async fn stats(
        &self,
        ctx: &Context<'_>,
        organization_id: Option<Uuid>,
        project_id: Option<Uuid>,
        collection_id: Option<Uuid>,
        measures: Option<Vec<Measure>>,
        granularity: Option<Granularity>,
        date_range: Option<DateRange>,
        order: Option<Order>,
        limit: Option<i32>,
    ) -> Result<Vec<DataPoint>> {
        let cube_client = ctx.data::<Client>()?;

        let time_dimension = {
            let granularity = granularity.map(|g| TimeGranularity::from(g).to_string());

            V1LoadRequestQueryTimeDimension {
                dimension: "mints.timestamp".to_string(),
                granularity,
                date_range: date_range.map(|dr| (dr.start_date, dr.end_date)), /* Convert DateRangeInput to tuple */
            }
        };
        let order = order.unwrap_or(Order::Desc).to_string();

        let measures: Vec<String> = measures
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|measure| format!("{}.{}", measure.resource, measure.operation))
            .collect();
        let dimension_by = match (organization_id, project_id, collection_id) {
            (Some(_), None, None) => "organization_id",
            (None, Some(_), Some(_)) => "project_id",
            _ => "collection_id",
        };

        let filter = V1LoadRequestQueryFilterItem::equals_member(
            &format!("mints.{dimension_by}"),
            collection_id.unwrap(),
        );

        let query = CubeQuery::new()
            .limit(limit.unwrap_or(100))
            .order("mints.timestamp", &order)
            .measures(measures)
            .dimensions(vec![&format!("mints.{dimension_by}")])
            .time_dimensions(time_dimension)
            .filter_member(filter);
        hub_core::tracing::info!("Query: {:#?}", query);

        let response = cube_client.execute_query(query).await?;

        let data_points = parse_data_points(&response)?;
        Ok(data_points)
    }
}

fn parse_data_points(response: &str) -> Result<Vec<DataPoint>> {
    let response_data: V1LoadResponse =
        serde_json::from_str(response).map_err(|e| async_graphql::Error::new(e.to_string()))?;

    let data: &Vec<serde_json::Value> = response_data
        .results
        .first()
        .ok_or_else(|| async_graphql::Error::new("No results found"))?
        .data
        .as_ref();

    let data_points: Result<Vec<DataPoint>, _> = data
        .iter()
        .map(|d| serde_json::from_value(d.clone()))
        .collect();

    data_points.map_err(|e| async_graphql::Error::new(e.to_string()))
}
