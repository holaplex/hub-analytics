use async_graphql::{Context, Object, Result};
use either::Either;
use hub_core::uuid::Uuid;

use crate::{
    cube_client::{Client, Query as CubeQuery},
    graphql::objects::{
        DataPoint, DataPoints, DateRange, Granularity, Measure, Order, TimeGranularity,
        V1LoadRequestQueryFilterItem as Filter, V1LoadRequestQueryTimeDimension as TimeDimension,
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
    /// * `dateRange` - DateFrom and DateTo, in YYYY-MM-DD format.
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
        measures: Option<Vec<Measure>>,
        date_range: Option<DateRange>,
        order: Option<Order>,
        limit: Option<i32>,
    ) -> Result<Vec<DataPoint>> {
        let cube_client = ctx.data::<Client>()?;

        let resource = measures.as_ref().and_then(|ms| ms.first()).map_or_else(
            || "mints".to_string(),
            |measure| measure.resource.to_string(),
        );

        let order = order.unwrap_or(Order::Desc).to_string();

        let measures: Vec<String> = measures
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|measure| format!("{res}.{op}", res = measure.resource, op = measure.operation))
            .collect();

        let (id, dimension) = get_id_and_dimension(organization_id, project_id, collection_id)?;

        let time_dimension = process_date_range(&resource, date_range)?;

        let filter = Filter::new()
            .member(&format!("{resource}.{dimension}"))
            .operator("equals")
            .values(vec![id]);

        let query = CubeQuery::new()
            .limit(limit.unwrap_or(100))
            .order(&format!("{resource}.timestamp"), &order)
            .measures(measures)
            .dimensions(vec![&format!("{resource}.{dimension}")])
            .time_dimensions(time_dimension)
            .filter_member(filter);

        hub_core::tracing::info!("Query: {:#?}", query);
        let data_points = DataPoints::try_from(cube_client.query(query).await?)?.into_vec();
        Ok(data_points)
    }
}

fn get_id_and_dimension(
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

fn process_date_range(
    resource: &str,
    range: Option<DateRange>,
) -> Result<TimeDimension, async_graphql::Error> {
    let mut td = TimeDimension::new(format!("{resource}.timestamp"));

    if let Some(range) = range {
        let granularity = match range.interval {
            Some(interval) => TimeGranularity::from(interval.to_granularity()),
            None => TimeGranularity::from(Granularity::Day),
        };

        td.granularity(&granularity.to_string());

        // Determine date range
        let dr = match (range.start, range.end) {
            (Some(start), Some(end)) => Either::Right(vec![start.to_string(), end.to_string()]),
            (None, None) if range.interval.is_some() => {
                Either::Left(range.interval.unwrap().to_string())
            },
            _ => {
                return Err(async_graphql::Error::new(
                    "Invalid DateRange provided. If start is defined, end must be defined too, and vice versa. If neither are defined, interval must be provided.",
                ));
            },
        };
        td.date_range(dr);
    }

    Ok(td)
}
