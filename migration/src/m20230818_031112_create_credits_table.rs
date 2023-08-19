use sea_orm_migration::prelude::*;

use crate::m20230804_212412_create_organizations_table::Organizations;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Credits::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Credits::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Credits::OrganizationId).uuid().not_null())
                    .col(ColumnDef::new(Credits::Amount).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-credits_organization_id-organizations")
                            .from(Credits::Table, Credits::OrganizationId)
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Credits::Timestamp).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("credits_organization_id_idx")
                    .table(Credits::Table)
                    .col(Credits::OrganizationId)
                    .index_type(IndexType::Hash)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Credits::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Credits {
    Table,
    Id,
    Amount,
    OrganizationId,
    Timestamp,
}
