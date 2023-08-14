use async_graphql::{ComplexObject, SimpleObject};
use hub_core::uuid::Uuid;

//use super::DataPoint;
//use crate::cube_client::Client;

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct Collection {
    pub id: Uuid,
}

#[ComplexObject]
impl Collection {}
