use async_graphql::{Context, Object, Result};
use hub_core::uuid::Uuid;

use crate::graphql::objects::Collection;

#[derive(Default)]
pub struct Query;

#[Object(name = "CollectionQuery")]
impl Query {
    #[graphql(entity)]
    async fn find_collection_by_id(
        &self,
        _ctx: &Context<'_>,
        #[graphql(key)] id: Uuid,
    ) -> Result<Collection> {
        Ok(Collection { id })
    }
}
