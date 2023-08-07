use sea_orm_migration::prelude::*;

use crate::{
    m20230804_212530_create_projects_table::Projects,
    m20230804_213809_create_collections_table::Collections,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Mints::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Mints::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Mints::Owner).string().not_null())
                    .col(ColumnDef::new(Mints::CollectionId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mints_collection_id-collections")
                            .from(Mints::Table, Mints::CollectionId)
                            .to(Collections::Table, Collections::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Mints::ProjectId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-mints_project_id-projects")
                            .from(Mints::Table, Mints::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Mints::Timestamp).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
            .name("mints_collection_id_idx")
            .table(Mints::Table)
            .col(Mints::CollectionId)
            .index_type(IndexType::Hash)
            .if_not_exists() // Adding this line to conditionally create the index
            .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Mints::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Mints {
    Table,
    Id,
    CollectionId,
    Owner,
    ProjectId,
    Timestamp,
}
