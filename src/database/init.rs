use tokio_postgres::Error;
use crate::database::database::DatabaseConnection;

pub fn create_tables() {
    create_table_manga()?;
    create_table_chapter()?;
}

fn create_table_manga() -> Result<(), Error>{
    let mut client = DatabaseConnection::client();
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS manga (
            id              SERIAL PRIMARY KEY,
            group_id        INTEGER NOT NULL,
            title           VARCHAR NOT NULL,
            description     VARCHAR NOT NULL,
            img             VARCHAR NOT NULL
        )
    ")
}

fn create_table_chapter() -> Result<(), Error>{
    let mut client = DatabaseConnection::client();
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS manga (
            id              SERIAL PRIMARY KEY,
            manga_id        INTEGER NOT NULL,
            translator_id   INTEGER NOT NULL,
            chapter_id      VARCHAR NOT NULL,
            link            VARCHAR NOT NULL
        )
    ")
}