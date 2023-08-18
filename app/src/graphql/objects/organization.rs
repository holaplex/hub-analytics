use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use hub_core::uuid::Uuid;

use crate::graphql::{
    objects::{DataPoint, Interval, Order},
    queries::analytics::Query,
};

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Organization {
    #[graphql(external)]
    pub id: Uuid,
}

#[ComplexObject]
impl Organization {
    async fn analytics(
        &self,
        ctx: &Context<'_>,
        interval: Option<Interval>,
        order: Option<Order>,
        limit: Option<i32>,
    ) -> Result<Vec<DataPoint>> {
        Query::analytics(
            &Query,
            ctx,
            Some(self.id),
            None,
            None,
            interval,
            order,
            limit,
        )
        .await
    }
}
