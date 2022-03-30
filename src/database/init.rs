use postgres::Error;
use crate::database::database::DatabaseConnection;

pub fn create_table_manga() -> Result<(), Error>{
    let mut client = DatabaseConnection::client();
    client.batch_execute("
        CREATE TABLE IF NOT EXISTS manga (
            id              SERIAL PRIMARY KEY,
            group_id        INTEGER NOT NULL,
            title           VARCHAR NOT NULL,
            description     VARCHAR NOT NULL,
            img             VARCHAR NOT NULL
        )
    ");

    Ok(())
}