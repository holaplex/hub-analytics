use async_graphql::{
    extensions::{ApolloTracing, Logger},
    EmptyMutation, EmptySubscription, Schema,
};

use crate::graphql::queries::Query;

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

/// Builds the GraphQL Schema, attaching the Database to the context
#[must_use]
pub fn build_schema() -> AppSchema {
    Schema::build(Query::default(), EmptyMutation, EmptySubscription)
        .extension(ApolloTracing)
        .extension(Logger)
        .enable_federation()
        .finish()
}
