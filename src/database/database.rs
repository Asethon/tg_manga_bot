use tokio_postgres::{Client, NoTls, Error};

pub struct DatabaseConnection {}

impl DatabaseConnection {
    pub async fn client() -> Result<Client, Error> {
        let login = dotenv::var("POSTGRES_LOGIN").unwrap();
        let db = dotenv::var("POSTGRES_DB").unwrap();
        let password = dotenv::var("POSTGRES_PASSWORD").unwrap();
        let host = dotenv::var("POSTGRES_HOST").unwrap();
        let params = format!("postgresql://{}:{}@{}/{}", login, password, host, db);
        let (client, connection) = tokio_postgres::connect(&*params, NoTls).await?;
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(client)
    }
}

pub trait Repository<T> {
    fn default() -> T {
        T{ client: Client, element: None }
    }

    fn get(&self) -> &T {
        self.T.as_ref().unwrap()
    }
}