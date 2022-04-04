#![allow(unused)]

use crate::database;
use tokio_postgres::{Client, Error};
use database::database::DatabaseConnection;

pub struct Chapter {
    pub(crate) id: Option<i32>,
    pub(crate) manga_id: i32,
    translator_id: i32,
    pub(crate) chapter_id: String,
    pub(crate) link: String,
}

pub struct ChapterRepository {
    client: Client,
    chapter: Option<Chapter>,
}

impl ChapterRepository {
    pub fn init(client: Client) -> ChapterRepository {
        ChapterRepository { client, chapter: None }
    }

    pub fn new(&mut self, manga_id: i32, translator_id: i32, chapter_id: String, link: String) -> &Self {
        self.chapter = Option::from(Chapter { id: None, manga_id, translator_id, chapter_id, link });
        self
    }

    pub fn get(&self) -> &Chapter {
        self.chapter.as_ref().unwrap()
    }

    pub fn set(&mut self, chapter: Chapter) -> &Self {
        self.chapter = Option::from(chapter);
        self
    }

    pub async fn push(&self) -> Result<(), Error> {
        let chapter = self.chapter.as_ref().unwrap();
        self.client.execute("INSERT INTO chapters (manga_id, translator_id, chapter_id, link) VALUES ($1, $2, $3, $4)",
                            &[&chapter.manga_id, &chapter.translator_id, &chapter.chapter_id, &chapter.link]).await?;
        Ok(())
    }

    pub async fn update(&self) -> Result<(), Error> {
        let chapter = self.chapter.as_ref().unwrap();
        self.client.execute("UPDATE chapters SET group_id=$1, title=$2, description=$3, img=$4",
                            &[&chapter.manga_id, &chapter.translator_id, &chapter.chapter_id, &chapter.link]).await?;

        Ok(())
    }

    pub async fn get_by_id(&self, id: i32) -> Result<Chapter, Error> {
        let chapter = self.client.query_one("SELECT * FROM chapters WHERE id=$1", &[&id]).await?;
        let id: i32 = chapter.get(0);
        Ok(Chapter {
            id: Option::from(id),
            manga_id: chapter.get(1),
            translator_id: chapter.get(2),
            chapter_id: chapter.get(3),
            link: chapter.get(4),
        })
    }

    pub async fn delete(&self) -> Result<(), Error> {
        match self.chapter.as_ref().unwrap().id {
            Some(id) => {
                self.client.execute("DELETE FROM chapters WHERE id=$1", &[&id]).await?;
                Ok(())
            }
            None => Ok(())
        }
    }

    pub async fn list(&self) -> Result<Vec<Chapter>, Error> {
        let mut chapter_list = vec![];
        for row in self.client.query("select * from chapters", &[]).await? {
            let id: i32 = row.get(0);
            chapter_list.push(Chapter {
                id: Option::from(id),
                manga_id: row.get(1),
                translator_id: row.get(2),
                chapter_id: row.get(3),
                link: row.get(4),
            });
        }
        Ok(chapter_list)
    }

    pub async fn list_by_manga_id(&self, id: i32) -> Result<Vec<Chapter>, Error> {
        let mut chapter_list = vec![];
        for row in self.client.query("select * from chapters WHERE manga_id=$1", &[&id]).await? {
            let id: i32 = row.get(0);
            chapter_list.push(Chapter {
                id: Option::from(id),
                manga_id: row.get(1),
                translator_id: row.get(2),
                chapter_id: row.get(3),
                link: row.get(4),
            });
        }
        Ok(chapter_list)
    }
}