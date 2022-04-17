use sea_schema::migration::prelude::*;
use entity::chapter;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20220101_000001_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_query::Table::create()
                    .table(chapter::Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(chapter::Column::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(chapter::Column::BookId).string().not_null())
                    .col(ColumnDef::new(chapter::Column::Translator_id).string().not_null())
                    .col(ColumnDef::new(chapter::Column::Description).string().not_null())
                    .col(ColumnDef::new(chapter::Column::Img).string().not_null())
                    .to_owned()
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                sea_query::Table::drop()
                    .table(book::Entity)
                    .to_owned()
            )
            .await
    }
}
