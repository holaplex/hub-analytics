use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use hub_core::uuid::Uuid;

use crate::graphql::{
    objects::{DataPoint, DateRange, Granularity, Measure, Order},
    queries::stats,
};

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Collection {
    #[graphql(external)]
    pub id: Uuid,
}

#[ComplexObject]
impl Collection {
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
        stats::Query::stats(
            &stats::Query,
            ctx,
            None,
            None,
            Some(self.id),
            measures,
            granularity,
            date_range,
            order,
            limit,
        )
        .await
    }
}
