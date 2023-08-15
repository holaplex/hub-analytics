use async_graphql::{Context, Object, Result};
use hub_core::uuid::Uuid;

use crate::graphql::objects::Project;

#[derive(Default)]
pub struct Query;

#[Object(name = "ProjectQuery")]
impl Query {
    #[graphql(entity)]
    async fn find_project_by_id(
        &self,
        _ctx: &Context<'_>,
        #[graphql(key)] id: Uuid,
    ) -> Result<Project> {
        Ok(Project { id })
    }
}
