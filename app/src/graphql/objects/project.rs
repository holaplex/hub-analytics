use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use hub_core::uuid::Uuid;

use crate::graphql::{
    objects::{DataPoint, DateRange, Order},
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
            date_range,
            order,
            limit,
        )
        .await
    }
}
