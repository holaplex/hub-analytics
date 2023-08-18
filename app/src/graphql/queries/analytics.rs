use std::collections::BTreeMap;

use async_graphql::{Context, Object, Result};
use either::Either;
use hub_core::{chrono::NaiveDateTime, uuid::Uuid};

use crate::{
    cube_client::{Client, Query as CubeQuery},
    graphql::objects::{
        DataPoint, DataPoints, Granularity, Interval, Measure, Operation, Order, Resource,
        TimeGranularity, V1LoadRequestQueryFilterItem as Filter,
        V1LoadRequestQueryTimeDimension as TimeDimension,
    },
};

#[derive(Debug, Clone, Default)]
pub struct Query;

#[Object(name = "AnalyticsQuery")]
impl Query {
    /// Returns a list of data points for a specific collection and timeframe.
    ///
    /// # Arguments
    /// * `organizationId` - The ID of the organization
    /// * `projectId` - The ID of the project.
    /// * `collectionId` - The ID of the collection.
    /// * `measures` - An map array of resources to query (resource, operation).
    /// * `interval` - The timeframe interval. `TODAY` | `YESTERDAY` | `THIS_MONTH` | `LAST_MONTH`
    /// * `order` - order the results by ASC or DESC.
    /// * `limit` - Optional limit on the number of data points to retrieve.
    ///
    /// # Returns
    /// A vector of Analytics objects representing the analytics data.
    ///
    /// # Errors
    /// This function returns an error if there was a problem with retrieving the data points.
    #[allow(clippy::too_many_arguments)]
    pub async fn analytics(
        &self,
        ctx: &Context<'_>,
        organization_id: Option<Uuid>,
        project_id: Option<Uuid>,
        collection_id: Option<Uuid>,
        interval: Option<Interval>,
        order: Option<Order>,
        limit: Option<i32>,
    ) -> Result<Vec<DataPoint>> {
        let cube = ctx.data::<Client>()?;

        let selections = Selection::from_context(ctx);

        let (id, root) = parse_id_and_root(organization_id, project_id, collection_id)?;

        let order = order.unwrap_or(Order::Desc);
        let mut datapoints = Vec::new();
        for selection in &selections {
            let resource = selection.resource.to_string();

            let mut time_dimension = TimeDimension::new(format!("{resource}.timestamp"));
            let granularity = match interval {
                Some(interval) => TimeGranularity::from(interval.to_granularity()),
                None => TimeGranularity::from(Granularity::Day),
            };

            time_dimension
                .granularity(&granularity.to_string())
                .date_range(Either::Left(interval.unwrap_or_default().to_string()));

            let filter = Filter::new()
                .member(&format!("{resource}.{root}"))
                .operator("equals")
                .values(vec![id.clone()]);

            let query = CubeQuery::new()
                .limit(limit.unwrap_or(100))
                .order(&format!("{resource}.timestamp"), &order.to_string())
                .measures(selection.measures.iter().map(Measure::as_string).collect())
                .dimensions(selection.dimensions.clone())
                .time_dimensions(time_dimension.clone())
                .filter_member(filter);

            hub_core::tracing::info!("Query: {query:#?}");

            datapoints.extend(
                DataPoints::from_response(&cube.query(query).await?, selection.resource)?
                    .into_vec(),
            );
        }
        let mut merged: BTreeMap<NaiveDateTime, DataPoint> = BTreeMap::new();
        for dp in &datapoints {
            if let Some(timestamp) = dp.timestamp {
                merged
                    .entry(timestamp)
                    .and_modify(|existing_dp: &mut DataPoint| existing_dp.merge(dp))
                    .or_insert_with(|| dp.clone());
            }
        }

        Ok(merged.into_values().collect())
    }
}

pub struct Selection {
    pub resource: Resource,
    pub measures: Vec<Measure>,
    pub dimensions: Vec<String>,
}

impl Selection {
    #[must_use]
    pub fn from_context(ctx: &Context<'_>) -> Vec<Selection> {
        let mut selections: Vec<Selection> = Vec::new();

        for field in ctx.field().selection_set() {
            if let Ok(resource) = field.name().parse::<Resource>() {
                let mut dimensions = Vec::new();
                let mut measures = Vec::new();
                for nested_field in field.selection_set() {
                    match nested_field.name() {
                        "count" => measures.push(Measure::new(resource, Operation::Count)),
                        "organizationId" => dimensions.push("projects.organization_id".to_string()),
                        "projectId" => dimensions.push(format!("{resource}.project_id")),
                        "collectionId" => dimensions.push(format!("{resource}.collection_id")),
                        _ => {},
                    }
                }

                let selection = Selection {
                    resource,
                    measures,
                    dimensions,
                };

                selections.push(selection);
            }
        }

        selections
    }
}

fn parse_id_and_root(
    organization_id: Option<Uuid>,
    project_id: Option<Uuid>,
    collection_id: Option<Uuid>,
) -> Result<(String, &'static str), async_graphql::Error> {
    match (organization_id, project_id, collection_id) {
        (Some(organization_id), None, None) => Ok((organization_id.to_string(), "organization_id")),
        (None, Some(project_id), None) => Ok((project_id.to_string(), "project_id")),
        (None, None, Some(collection_id)) => Ok((collection_id.to_string(), "collection_id")),
        _ => Err(async_graphql::Error::new(
            "No valid [project,organization,collection] ID or multiple IDs provided",
        )),
    }
}
