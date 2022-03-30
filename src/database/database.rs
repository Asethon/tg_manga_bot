use postgres::{Client, NoTls};

pub struct DatabaseConnection {
    client: Client,
}

impl DatabaseConnection {
    pub fn client() -> Client {
        let login = dotenv::var("POSTGRES_LOGIN").unwrap();
        let password = dotenv::var("POSTGRES_PASSWORD").unwrap();
        let host = dotenv::var("POSTGRES_HOST").unwrap();
        let params = format!("postgresql://{}:{}@{}", login, password, host);
        let mut client = Client::connect(&params, NoTls);

        client.unwrap()
    }
}