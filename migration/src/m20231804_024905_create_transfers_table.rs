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
                    .table(Transfers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transfers::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Transfers::CollectionId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transfers_collection_id-collections")
                            .from(Transfers::Table, Transfers::CollectionId)
                            .to(Collections::Table, Collections::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Transfers::ProjectId).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transfers_project_id-projects")
                            .from(Transfers::Table, Transfers::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(ColumnDef::new(Transfers::Timestamp).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                IndexCreateStatement::new()
                    .name("transfers_collection_id_idx")
                    .table(Transfers::Table)
                    .col(Transfers::CollectionId)
                    .index_type(IndexType::Hash)
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transfers::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Transfers {
    Table,
    Id,
    CollectionId,
    ProjectId,
    Timestamp,
}