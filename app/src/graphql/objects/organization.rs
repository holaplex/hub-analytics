use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use hub_core::uuid::Uuid;

use crate::graphql::objects::{Interval, Order};

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
        _ctx: &Context<'_>,
        _interval: Option<Interval>,
        _order: Option<Order>,
        _limit: Option<i32>,
    ) -> Result<String> {
        Ok(String::new())
    }
}
