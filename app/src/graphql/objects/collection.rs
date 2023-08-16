use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use hub_core::uuid::Uuid;

use crate::graphql::{
    objects::{DataPoint, DateRange, Measure, Order},
    queries::analytics::Query,
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
    async fn analytics(
        &self,
        ctx: &Context<'_>,
        measures: Option<Vec<Measure>>,
        date_range: Option<DateRange>,
        order: Option<Order>,
        limit: Option<i32>,
    ) -> Result<Vec<DataPoint>> {
        Query::analytics(
            &Query,
            ctx,
            None,
            None,
            Some(self.id),
            measures,
            date_range,
            order,
            limit,
        )
        .await
    }
}
