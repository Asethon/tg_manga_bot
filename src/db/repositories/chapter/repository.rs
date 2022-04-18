use sea_orm::entity::prelude::*;
use sea_orm::entity::*;

use crate::domain;
use crate::domain::chapters::chapter;

pub struct Repository {
    pub(crate) db: DatabaseConnection,
}

impl Repository {
    pub async fn insert(
        &self,
        book_id: i32,
        translator_id: i32,
        chapter_id: String,
        link: String,
    ) {
        let chapter = chapter::ActiveModel {
            book_id: Set(book_id),
            translator_id: Set(translator_id),
            chapter_id: Set(chapter_id),
            link: Set(link),
            ..Default::default()
        };
        domain::Chapter::insert(chapter)
            .exec(&self.db)
            .await
            .expect("could not insert chapter");
    }

    pub async fn find_by_id(&self, id: i32) -> chapter::Model {
        let chapter: Option<chapter::Model> = domain::Chapter::find_by_id(id).one(&self.db).await.unwrap();
        chapter.unwrap()
    }

    pub async fn find_by_book_id(&self, book_id: i32) -> Vec<chapter::Model> {
        let chapter = domain::Chapter::find()
            .filter(chapter::Column::BookId.eq(book_id))
            .all(&self.db)
            .await.unwrap();
        chapter
    }

    pub async fn delete(&self, id: i32) {
        domain::Chapter::delete_by_id(id).exec(&self.db).await;
    }
}