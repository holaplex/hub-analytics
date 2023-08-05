pub use sea_orm_migration::prelude::*;

mod m20230804_212412_create_organizations_table;
mod m20230804_212530_create_projects_table;
mod m20230804_212603_create_customers_table;
mod m20230804_213809_create_collections_table;
mod m20230804_214701_create_mints_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230804_212412_create_organizations_table::Migration),
            Box::new(m20230804_212530_create_projects_table::Migration),
            Box::new(m20230804_212603_create_customers_table::Migration),
            Box::new(m20230804_213809_create_collections_table::Migration),
            Box::new(m20230804_214701_create_mints_table::Migration),
        ]
    }
}
