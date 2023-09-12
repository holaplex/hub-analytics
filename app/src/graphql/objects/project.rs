use async_graphql::{ComplexObject, Context, Result, SimpleObject};
use hub_core::uuid::Uuid;

use crate::{
    cube_client::{Client, Query},
    entities::{customers, mints},
    graphql::objects::{
        CustomerDataPoint, Interval, MintDataPoint, Order, V1LoadRequestQueryFilterItem as Filter,
        V1LoadRequestQueryTimeDimension as TimeDimension,
    },
};

#[derive(Debug, Clone, SimpleObject)]
#[graphql(complex)]
pub struct ProjectAnalytics {
    id: Uuid,
    interval: Option<Interval>,
    order: Option<Order>,
    limit: Option<i32>,
}

impl ProjectAnalytics {
    pub fn new(
        id: Uuid,
        interval: Option<Interval>,
        order: Option<Order>,
        limit: Option<i32>,
    ) -> Self {
        Self {
            id,
            interval,
            order,
            limit,
        }
    }
}

#[ComplexObject]
impl ProjectAnalytics {
    #[allow(clippy::too_many_arguments)]
    async fn mints(
        &self,
        ctx: &Context<'_>,
        dimensions: Option<Vec<mints::Dimension>>,
    ) -> Result<Vec<MintDataPoint>> {
        let time_dimension = ctx.look_ahead().field("timestamp").exists().then(|| {
            let interval = self.interval.unwrap_or_default();

            TimeDimension::new("mints.timestamp".to_string())
                .date_range(interval.to_date_range())
                .granularity(&interval.to_granularity().to_string())
                .clone()
        });

        let cube = ctx.data::<Client>()?;

        let filter = Filter::new()
            .member("mints.project_id")
            .operator("equals")
            .values(vec![self.id.to_string()]);

        let mut query = Query::new()
            .limit(self.limit.unwrap_or(100))
            .measures(vec!["mints.count".to_string()])
            .dimensions(dimensions.map_or(vec![], |dimensions| {
                dimensions
                    .into_iter()
                    .map(|dimension| dimension.to_string())
                    .collect()
            }))
            .filter_member(filter);

        if time_dimension.is_some() {
            query = query.time_dimensions(time_dimension);
        }

        let results = cube.query::<MintDataPoint>(query).await?;

        Ok(results)
    }

    async fn customers(
        &self,
        ctx: &Context<'_>,
        dimensions: Option<Vec<customers::Dimension>>,
    ) -> Result<Vec<CustomerDataPoint>> {
        let time_dimension = ctx.look_ahead().field("timestamp").exists().then(|| {
            let interval = self.interval.unwrap_or_default();

            TimeDimension::new("customers.timestamp".to_string())
                .date_range(interval.to_date_range())
                .granularity(&interval.to_granularity().to_string())
                .clone()
        });

        let cube = ctx.data::<Client>()?;

        let filter = Filter::new()
            .member("customers.project_id")
            .operator("equals")
            .values(vec![self.id.to_string()]);

        let mut query = Query::new()
            .limit(self.limit.unwrap_or(100))
            .dimensions(dimensions.map_or(vec![], |dimensions| {
                dimensions
                    .into_iter()
                    .map(|dimension| dimension.to_string())
                    .collect()
            }))
            .measures(vec!["customers.count".to_string()])
            .filter_member(filter);

        if time_dimension.is_some() {
            query = query.time_dimensions(time_dimension);
        }

        let results = cube.query::<CustomerDataPoint>(query).await?;

        Ok(results)
    }
}

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
        _ctx: &Context<'_>,
        interval: Option<Interval>,
        order: Option<Order>,
        limit: Option<i32>,
    ) -> Result<ProjectAnalytics> {
        Ok(ProjectAnalytics::new(self.id, interval, order, limit))
    }
}
