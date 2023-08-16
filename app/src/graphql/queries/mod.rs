#![allow(clippy::unused_async)]

pub mod analytics;
mod collection;
mod organization;
mod project;

// // Add your other ones here to create a unified Query object
#[derive(async_graphql::MergedObject, Default)]
pub struct Query(
    analytics::Query,
    organization::Query,
    project::Query,
    collection::Query,
);
