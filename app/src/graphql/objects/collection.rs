use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use hub_core::uuid::Uuid;

use crate::{
    cube_client::{Client, Query as CubeQuery},
    graphql::objects::{
        DataPoint, DataPoints, DateRange, Granularity, Measure, Order, TimeGranularity,
        V1LoadRequestQueryFilterItem, V1LoadRequestQueryTimeDimension,
    },
};
#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Collection {
    pub id: Uuid,
}

#[ComplexObject]
impl Collection {
    /// # Returns
    /// A vector of Stats objects representing the analytics data.
    ///
    /// # Errors
    /// This function returns an error if there was a problem with retrieving the data points.
    #[allow(clippy::too_many_arguments)]
    async fn stats(
        &self,
        ctx: &Context<'_>,
        measures: Option<Vec<Measure>>,
        granularity: Option<Granularity>,
        date_range: Option<DateRange>,
        order: Option<Order>,
        limit: Option<i32>,
    ) -> Result<Vec<DataPoint>> {
        let cube_client = ctx.data::<Client>()?;

        let target_resource = measures.as_ref().and_then(|ms| ms.first()).map_or_else(
            || "mints".to_string(),
            |measure| measure.resource.to_string(),
        );
        let time_dimension = {
            let granularity = granularity.map(|g| TimeGranularity::from(g).to_string());

            V1LoadRequestQueryTimeDimension {
                dimension: format!("{target_resource}.timestamp"),
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

        let dimension = "collection_id";

        let filter = V1LoadRequestQueryFilterItem::equals_member(
            &format!("{target_resource}.{dimension}"),
            self.id,
        );

        let query = CubeQuery::new()
            .limit(limit.unwrap_or(100))
            .order(&format!("{target_resource}.timestamp"), &order)
            .measures(measures)
            .dimensions(vec![&format!("{target_resource}.{dimension}")])
            .time_dimensions(time_dimension)
            .filter_member(filter);

        hub_core::tracing::info!("Query: {:#?}", query);

        let response = cube_client.execute_query(query).await?;
        let data_points: Vec<DataPoint> = DataPoints::try_from(response)?.into_vec();
        Ok(data_points)
    }
}
