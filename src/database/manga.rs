#![allow(unused)]

use crate::database;
use tokio_postgres::{Client, Error};
use database::database::DatabaseConnection;

pub struct Manga {
    pub(crate) id: Option<i32>,
    pub(crate) group_id: i32,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) img: String,
}

pub struct MangaRepository {
    client: Client,
    manga: Option<Manga>,
}

impl Default for MangaRepository {
    fn default() -> Self {
        let client = DatabaseConnection::client().await?;
        MangaRepository { client, manga: None }
    }
}

impl MangaRepository  {
    pub fn init(client: Client) -> Self {
        MangaRepository { client, manga: None }
    }

    pub fn new(&mut self, group_id: i32, title: String, description: String, img: String) -> &Self {
        self.manga = Option::from(Manga { id: None, group_id, title, description, img });
        self
    }

    pub fn get(&mut self) -> &Manga {
        self.manga.as_ref().unwrap()
    }

    pub fn set(&mut self, manga: Manga) -> &Self {
        self.manga = Option::from(manga);
        self
    }

    pub async fn push(&mut self) -> Result<(), Error> {
        let manga = self.manga.as_ref().unwrap();
        self.client.execute("INSERT INTO manga (), VALUES ($1, $2, $3, $4)",
                            &[&manga.group_id, &manga.title, &manga.description, &manga.img]).await?;
        Ok(())
    }

    pub async fn update(&mut self) -> Result<(), Error> {
        let manga = self.manga.as_ref().unwrap();
        self.client.execute("UPDATE manga SET group_id=$1, title=$2, description=$3, img=$4",
                            &[&manga.group_id, &manga.title, &manga.description, &manga.img]
        ).await?;

        Ok(())
    }

    pub async fn get_by_id(&mut self, id: i32) -> Result<Manga, Error> {
        let manga = self.client.query_one("SELECT * FROM manga WHERE id=$1", &[&id]).await?;
        let id: i32 = manga.get(0);
        Ok(Manga {
            id: Option::from(id),
            group_id: manga.get(1),
            title: manga.get(2),
            description: manga.get(3),
            img: manga.get(4),
        })
    }

    pub async fn delete(&mut self) -> Result<(), Error> {
        match self.manga.as_ref().unwrap().id {
            Some(id) => {
                self.client.execute("DELETE FROM manga WHERE id=$1", &[&id]).await?;
                Ok(())
            }
            None => Ok(())
        }
    }

    pub async fn list(&mut self) -> Result<Vec<Manga>, Error> {
        let mut manga_list = vec![];
        for row in self.client.query("select * from manga", &[]).await? {
            manga_list.push(Manga {
                id: row.get(0),
                group_id: row.get(1),
                title: row.get(2),
                description: row.get(3),
                img: row.get(4),
            });
        }
        Ok(manga_list)
    }
}