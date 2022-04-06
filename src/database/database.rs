use tokio_postgres::{Client, NoTls, Error};
use crate::database::manga::Manga;

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

pub trait RepositoryS{}

pub struct RepositoryStruct<T> {
    client: Client,
    element: Option<T>,
}

impl RepositoryS for RepositoryStruct<Manga> {

}

pub trait Repository<T, R>
where
R: RepositoryS
{
    fn new(client: Client) -> R {
        RepositoryStruct { client, element: None }
    }

    fn get(&self) -> &T
    where
    Self: RepositoryS
    {
        self.element.as_ref().unwrap()
    }

    fn set(&mut self, element: T) -> R
    where
    Self: RepositoryS
    {
        self.element = Option::from(element);
        self
    }
}