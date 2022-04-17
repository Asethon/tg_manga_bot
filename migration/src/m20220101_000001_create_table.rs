use sea_schema::migration::prelude::*;
use entity::book;

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
                    .table(book::Entity)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(book::Column::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(book::Column::BookType).string().not_null())
                    .col(ColumnDef::new(book::Column::Title).string().not_null())
                    .col(ColumnDef::new(book::Column::Description).string().not_null())
                    .col(ColumnDef::new(book::Column::Img).string().not_null())
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
