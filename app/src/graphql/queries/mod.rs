#![allow(clippy::unused_async)]

mod collection;
mod stats;

// // Add your other ones here to create a unified Query object
#[derive(async_graphql::MergedObject, Default)]
pub struct Query(stats::Query, collection::Query);
