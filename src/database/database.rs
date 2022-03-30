use tokio_postgres::{Client, NoTls};

pub struct DatabaseConnection {
    client: Client,
}

impl DatabaseConnection {
    pub async fn client() -> Client {
        let login = dotenv::var("POSTGRES_LOGIN").unwrap();
        let password = dotenv::var("POSTGRES_PASSWORD").unwrap();
        let host = dotenv::var("POSTGRES_HOST").unwrap();
        let params = format!("postgresql://{}:{}@{}", login, password, host);
        let (client, connection) = tokio_postgres::connect(&*params, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });
        client.unwrap()
    }
}