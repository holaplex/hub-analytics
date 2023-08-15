use async_graphql::{Context, Object, Result};
use hub_core::uuid::Uuid;

use crate::{
    cube_client::{Client, Query as CubeQuery, TimeGranularity},
    graphql::objects::{
        DataPoint, DataPoints, DateRange, Granularity, Measure, Order,
        V1LoadRequestQueryFilterItem, V1LoadRequestQueryTimeDimension,
    },
};

#[derive(Debug, Clone, Default)]
pub struct Query;

#[Object(name = "StatQuery")]
impl Query {
    /// Returns a list of data points for a specific collection and timeframe.
    ///
    /// # Arguments
    /// * `organizationId` - The ID of the organization
    /// * `projectId` - The ID of the project.
    /// * `collectionId` - The ID of the collection.
    /// * `measures` - An map array of resources to query (resource, operation).
    /// * `granularity` - The time granularity for grouping (e.g., Day, Week, Month, Year).
    /// * `dateRange` - DateFrom and DateTo, in YYYY-MM-DD format.
    /// * `order` - order the results by ASC or DESC.
    /// * `limit` - Optional limit on the number of data points to retrieve.
    ///
    /// # Returns
    /// A vector of Stats objects representing the analytics data.
    ///
    /// # Errors
    /// This function returns an error if there was a problem with retrieving the data points.
    #[allow(clippy::too_many_arguments)]
    pub async fn stats(
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

        let resource = measures.as_ref().and_then(|ms| ms.first()).map_or_else(
            || "mints".to_string(),
            |measure| measure.resource.to_string(),
        );
        let time_dimension = {
            let granularity = granularity.map(|g| TimeGranularity::from(g).to_string());

            V1LoadRequestQueryTimeDimension {
                dimension: format!("{resource}.timestamp"),
                granularity,
                date_range: date_range.map(|dr| (dr.start_date, dr.end_date)),
            }
        };
        let order = order.unwrap_or(Order::Desc).to_string();

        let measures: Vec<String> = measures
            .as_ref()
            .unwrap_or(&Vec::new())
            .iter()
            .map(|measure| format!("{}.{}", measure.resource, measure.operation))
            .collect();

        let dimension = match (organization_id, project_id, collection_id) {
            (Some(_), None, None) => "organization_id",
            (None, Some(_), Some(_)) => "project_id",
            _ => "collection_id",
        };

        let filter = V1LoadRequestQueryFilterItem::equals_member(
            &format!("{resource}.{dimension}",),
            collection_id.unwrap(),
        );

        let query = CubeQuery::new()
            .limit(limit.unwrap_or(100))
            .order(&format!("{resource}.timestamp"), &order)
            .measures(measures)
            .dimensions(vec![&format!("{resource}.{dimension}")])
            .time_dimensions(time_dimension)
            .filter_member(filter);

        hub_core::tracing::info!("Query: {:#?}", query);

        let response = cube_client.execute_query(query).await?;
        let data_points: Vec<DataPoint> = DataPoints::try_from(response)?.into_vec();
        Ok(data_points)
    }
}
