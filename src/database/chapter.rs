use crate::database;
use postgres::{Client, Error};
use database::database::DatabaseConnection;

struct Chapter {
    pub(crate) id: Option<i32>,
    pub(crate) manga_id: i32,
    translator_id: i32,
    pub(crate) chapter_id: &'static str,
    pub(crate) link: &'static str,
}

pub struct ChapterRepository {
    client: Client,
    chapter: Option<Chapter>,
}

impl Default for ChapterRepository {
    fn default() -> Self {
        let mut client = DatabaseConnection::client();
        ChapterRepository { client, chapter: None }
    }
}

impl ChapterRepository {
    pub fn new(&mut self, manga_id: i32, translator_id: i32, chapter_id: &str, link: &str) -> &Self {
        self.chapter = Option::from(Chapter { id: None, manga_id, translator_id, chapter_id, link });
        self
    }

    pub fn get(&mut self) -> Chapter {
        self.chapter.unwrap()
    }

    pub fn set(&mut self, chapter: Chapter) -> &Self {
        self.chapter = Option::from(chapter);
        self
    }

    pub fn push(&mut self) -> Result<(), Error> {
        let chapter = self.chapter.unwrap();
        self.client.execute("INSERT INTO chapters (), VALUES ($1, $2, $3, $4, $5)",
                            &[chapter.manga_id, chapter.translator_id, chapter.chapter_id, chapter.link]);
        Ok(())
    }

    pub fn update(&mut self) -> Result<(), Error> {
        let chapter = self.chapter.unwrap();
        self.client.execute("UPDATE chapters SET group_id=$1, title=$2, description=$3, img=$4",
                            &[chapter.manga_id, chapter.translator_id, chapter.chapter_id, chapter.link],
        );

        Ok(())
    }

    pub fn get_by_id(&mut self, id: i32) -> Result<Chapter, Error> {
        let chapter = self.client.query_one("SELECT * FROM chapters WHERE id=$1", &[id])?;

        Ok(Manga {
            id: Option::from(chapter.get(0)),
            group_id: chapter.get(1),
            title: chapter.get(2),
            description: chapter.get(3),
            img: chapter.get(4),
        })
    }

    pub fn delete(&mut self) -> Result<(), Error> {
        match self.chapter.unwrap().id {
            Some(id) => {
                self.client.execute("DELETE FROM chapters WHERE id=$1", &[id]);
                Ok(())
            }
            None => Error
        }
    }

    pub fn list(&mut self) -> Result<Vec<Chapter>, Error> {
        let mut chapter_list = vec![];
        for row in self.client.query("select * from chapters", &[])? {
            chapter_list.push(Chapter {
                id: row.get(0),
                manga_id: row.get(1),
                translator_id: row.get(2),
                chapter_id: row.get(3),
                link: row.get(4),
            });
        }
        Ok(chapter_list)
    }

    pub fn list_by_manga_id(&mut self, id: i32) -> Result<Vec<Chapter>, Error> {
        let mut chapter_list = vec![];
        for row in self.client.query("select * from chapters WHERE manga_id=$1", &[id])? {
            chapter_list.push(Chapter {
                id: row.get(0),
                manga_id: row.get(1),
                translator_id: row.get(2),
                chapter_id: row.get(3),
                link: row.get(4),
            });
        }
        Ok(chapter_list)
    }
}