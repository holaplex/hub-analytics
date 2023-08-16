use sea_orm_migration::prelude::*;

use crate::{
    m20230804_212530_create_projects_table::Projects,
    m20230804_212603_create_customers_table::Customers,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Wallets::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Wallets::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Wallets::ProjectId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-wallets_project_id-projects")
                            .from(Wallets::Table, Wallets::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Wallets::CustomerId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-wallets_customer_id-customers")
                            .from(Wallets::Table, Wallets::CustomerId)
                            .to(Customers::Table, Customers::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Wallets::Blockchain).string().not_null())
                    .col(ColumnDef::new(Wallets::Timestamp).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("wallets_project_id_idx")
                    .table(Wallets::Table)
                    .col(Wallets::ProjectId)
                    .index_type(IndexType::Hash)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Wallets::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Wallets {
    Table,
    Id,
    CustomerId,
    ProjectId,
    Blockchain,
    Timestamp,
}
