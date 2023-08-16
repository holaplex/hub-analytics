use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use hub_core::uuid::Uuid;

use crate::graphql::{
    objects::{DataPoint, DateRange, Measure, Order},
    queries::analytics::Query,
};

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Project {
    #[graphql(external)]
    pub id: Uuid,
}

#[ComplexObject]
impl Project {
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
            Some(self.id),
            None,
            measures,
            date_range,
            order,
            limit,
        )
        .await
    }
}
