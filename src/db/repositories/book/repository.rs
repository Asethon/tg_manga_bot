use sea_orm::entity::prelude::*;
use sea_orm::entity::*;

use crate::domain;
use crate::domain::books::book;

pub struct Repository {
    pub(crate) db: DatabaseConnection,
}

impl Repository {
    pub async fn insert(
        &self,
        book_type: book::BookType,
        title: String,
        description: String,
        img: String,
    ) {
        let book = book::ActiveModel {
            book_type: Set(book_type),
            title: Set(title),
            description: Set(description),
            img: Set(img),
            ..Default::default()
        };
        domain::Book::insert(book)
            .exec(&self.db)
            .await
            .expect("could not insert book");
    }

    pub async fn find_by_id(&self, id: i32) -> book::Model {
        let book: Option<book::Model> = domain::Book::find_by_id(id).one(&self.db).await
            .unwrap_or(book::Model {
                id: 0,
                book_type: book::BookType::Manga,
                title: "Not Found".to_string(),
                description: "???".to_string(),
                img: "None".to_string()
            });
        book.unwrap()
    }

    pub async fn find_by_title(&self, title: &str) -> book::Model {
        let book: Option<book::Model> = domain::Book::find()
            .filter(book::Column::Title.contains(title))
            .one(&self.db)
            .await
            .unwrap_or(book::Model {
                id: 0,
                book_type: book::BookType::Manga,
                title: "Not Found".to_string(),
                description: "???".to_string(),
                img: "None".to_string()
            });
        book.unwrap()
    }

    pub async fn find_by_filter(&self) -> Vec<book::Model> {
        let books = domain::Book::find()
            .all(&self.db)
            .await.unwrap();
        books
    }

    pub async fn delete(&self, id: i32) {
        domain::Book::delete_by_id(id).exec(&self.db).await;
    }
}