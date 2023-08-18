use sea_orm_migration::prelude::*;

use crate::{
    m20230804_212412_create_organizations_table::Organizations,
    m20230804_212530_create_projects_table::Projects,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Webhooks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Webhooks::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Webhooks::OrganizationId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-webhooks_organization_id-organizations")
                            .from(Webhooks::Table, Webhooks::ProjectId)
                            .to(Organizations::Table, Organizations::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Webhooks::ProjectId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-webhooks_project_id-projects")
                            .from(Webhooks::Table, Webhooks::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Webhooks::Timestamp).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("webhooks_project_id_idx")
                    .table(Webhooks::Table)
                    .col(Webhooks::OrganizationId)
                    .index_type(IndexType::Hash)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Webhooks::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum Webhooks {
    Table,
    Id,
    ProjectId,
    OrganizationId,
    Timestamp,
}
