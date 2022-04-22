use sea_orm::DatabaseConnection;
use teloxide::Bot;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup,
        ParseMode::MarkdownV2,
    },
    utils::command::BotCommands,
    dispatching::dialogue::InMemStorage,
};
use teloxide::types::{KeyboardButton, KeyboardMarkup};

pub mod db;
pub mod domain;

use db::migrations;
use crate::db::{BookRepository, ChapterRepository};
use crate::domain::books::book::BookType;


#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = ":)")]
    Help,
    #[command(description = "Start")]
    Start,
    #[command(description = "Ping-pong")]
    Ping,

    #[command(description = "Добавить произведение")]
    BookAdd,

    #[command(description = "Добавить произведение")]
    ChapterAdd { id: i32 },

    #[command(description = "Главное меню")]
    Menu,
}

fn make_key() -> KeyboardMarkup {
    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];
    keyboard.push(vec![KeyboardButton::new("Button")]);
    KeyboardMarkup::new(keyboard)
}

async fn make_keyboard(book_id: Option<i32>) -> KeyboardButton {
    let db = get_db().await;
    let mut row = vec![];
    let mut keyboard: Vec<Vec<KeyboardButton>> = vec![];

    match book_id {
        Some(id) => {
            let repository = ChapterRepository { db: db.clone() };
            let chapters = repository.find_by_book_id(id).await;
            row = chapters
                .into_iter()
                .map(|chapter| {
                    let ch = format!("Глава: {}", chapter.chapter_id);
                    let link = format!("/chapter?{}", chapter.chapter_id);
                    KeyboardButton::callback(ch, link)
                })
                .collect();
            let link = format!("/chapter?{}", id);
            row.push(KeyboardButton::callback(
                "Добавить".to_owned(),
                link,
            ));
        }
        None => {
            let repository = BookRepository { db: db.clone() };
            let books = repository.find_by_filter().await;
            row = books
                .into_iter()
                .map(|book| {
                    let link = format!("/book?{}", book.id);
                    KeyboardButton::callback(
                        book.title.clone(),
                        link,
                    )
                })
                .collect();
            row.push(KeyboardButton::callback(
                "Добавить".to_owned(),
                "/book_add".to_owned(),
            ));
        }
    }

    keyboard.push(row);
    KeyboardButton::new(keyboard)
}

async fn message_handler(
    m: Message,
    bot: AutoSend<Bot>,
    dialogue: BookDialogue,
    command: Command,
) -> anyhow::Result<()> {
    match command {
        Command::Help => {
            // Just send the description of all commands.
            bot.send_message(m.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Start => {
            bot.send_message(m.chat.id, "Hi, send me /menu").reply_markup(make_key()).await?;
        }

        Command::Menu => {
            let keyboard = make_keyboard(None).await;
            bot.send_message(m.chat.id, "Каталог:").reply_markup(keyboard).await?;
        }

        Command::BookAdd => {
            bot.send_message(m.chat.id, "Введите название произведения: ").await?;
            dialogue.update(State::AddBookTitle).await?;
        }

        Command::ChapterAdd { id } => {
            bot.send_message(m.chat.id, "Номер главы:").await?;
            dialogue.update(State::AddChapterId { book_id: id }).await?;
        }

        Command::Ping => {
            bot.send_message(m.chat.id, "pong").await?;
        }
    }

    Ok(())
}

async fn callback_handler(
    q: CallbackQuery,
    bot: AutoSend<Bot>,
    dialogue: BookDialogue,
) -> anyhow::Result<()> {
    if let Some(link) = q.data {
        match q.message {
            Some(Message { id, chat, .. }) => {
                let split: Vec<&str> = link.split('?').collect();
                let link_id: Option<i32> = if split.len() >= 2 {
                    Some(split[1].parse::<i32>().unwrap())
                } else {
                    None
                };
                match split[0] {
                    "/book" => {
                        match link_id {
                            None => {
                                bot.send_message(chat.id, "Error").await?;
                            }
                            Some(_) => {
                                let keyboard = make_keyboard(Some(link_id.unwrap())).await;
                                bot.send_message(chat.id, "Главы:")
                                    .reply_markup(keyboard).await?;
                            }
                        }
                    }
                    "/chapter" => {
                        match link_id {
                            None => {
                                bot.send_message(chat.id, "Error").await?;
                            }
                            Some(_) => {
                                let url = dotenv::var("DATABASE_URL").unwrap();
                                let db = sea_orm::Database::connect(url).await.unwrap();
                                let repository = ChapterRepository { db };
                                let chapter = repository.find_by_id(link_id.unwrap()).await;
                                let keyboard = make_keyboard(Some(chapter.book_id)).await;
                                let link = format!("[Глава {}]({})", chapter.id, chapter.link);
                                bot.edit_message_text(chat.id, id, link)
                                    .reply_markup(keyboard)
                                    .parse_mode(MarkdownV2)
                                    .await?;
                            }
                        }
                    }
                    "/book_add" => {
                        bot.send_message(chat.id, "Введите название произведения: ").await?;
                        dialogue.update(State::AddBookTitle).await?;
                    }
                    _ => {}
                }
            }
            None => {}
        }
    }

    Ok(())
}

type BookDialogue = Dialogue<State, InMemStorage<State>>;


async fn get_db() -> DatabaseConnection {
    let url = dotenv::var("DATABASE_URL").unwrap();
    let db = sea_orm::Database::connect(url).await.unwrap();
    db
}

async fn add_book_title_handler(
    bot: AutoSend<Bot>,
    m: Message,
    dialogue: BookDialogue,
) -> anyhow::Result<()> {
    match m.text() {
        None => {
            dialogue.update(State::Start).await?;
        },
        Some(title) => {
            bot.send_message(m.chat.id, "Тип произведения (manga, ranobe):").await?;
            dialogue.update(State::AddBookType { title: title.into() }).await?;
        }
    }
    Ok(())
}

async fn add_book_type_handler(
    bot: AutoSend<Bot>,
    m: Message,
    dialogue: BookDialogue,
    (title, ): (String, ),
) -> anyhow::Result<()> {
    match m.text() {
        None => {
            dialogue.update(State::Start).await?;
        },
        Some(book_type) => {
            bot.send_message(m.chat.id, "Описание произведения:").await?;
            dialogue.update(State::AddBookDescription { title: title.into(), book_type: book_type.into() }).await?;
        }
    }
    Ok(())
}

async fn add_book_description_handler(
    bot: AutoSend<Bot>,
    m: Message,
    dialogue: BookDialogue,
    (title, book_type): (String, String),
) -> anyhow::Result<()> {
    match m.text() {
        None => {
            dialogue.update(State::Start).await?;
        },
        Some(description) => {
            let db = get_db().await;
            let repository = BookRepository { db };
            repository.insert(
                BookType::try_from(&*book_type).unwrap(),
                title,
                description.into(),
                "None".into(),
            ).await;
            bot.send_message(m.chat.id, "Произведение добавлено").await?;
            dialogue.update(State::Start).await?;
        }
    }
    Ok(())
}

async fn add_chapter_id_handler(
    bot: AutoSend<Bot>,
    m: Message,
    dialogue: BookDialogue,
    (book_id, ): (i32, ),
) -> anyhow::Result<()> {
    match m.text() {
        None => {
            dialogue.update(State::Start).await?;
        },
        Some(chapter_id) => {
            bot.send_message(m.chat.id, "Ссылка:").await?;
            dialogue.update(State::AddChapterLink { book_id, chapter_id: chapter_id.to_string() }).await?;
        }
    }
    Ok(())
}

async fn add_chapter_link_handler(
    bot: AutoSend<Bot>,
    m: Message,
    dialogue: BookDialogue,
    (book_id, chapter_id): (i32, String),
) -> anyhow::Result<()> {
    match m.text() {
        None => {
            dialogue.update(State::Start).await?;
        },
        Some(link) => {
            let user_id = bot.get_me().await?.user.id.0 as i32;
            let db = get_db().await;
            let repository = ChapterRepository { db };
            repository.insert(book_id, user_id, chapter_id.clone(), link.to_string()).await;
            let text = format!("Глава {} добавлена.", chapter_id);
            bot.send_message(m.chat.id, text).await?;
            dialogue.update(State::Start).await?;
        }
    }
    Ok(())
}

#[derive(Clone)]
pub enum State {
    Start,
    ///Book
    AddBookTitle,
    AddBookType { title: String },
    AddBookDescription { title: String, book_type: String },
    ///Chapter
    AddChapterId { book_id: i32 },
    AddChapterLink { book_id: i32, chapter_id: String },
}

impl Default for State {
    fn default() -> Self {
        Self::Start
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    up_database();
    let bot = Bot::from_env().auto_send();

    let handler = dptree::entry()
        .branch(Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .filter_command::<Command>()
            .branch(teloxide::handler![State::Start].endpoint(message_handler))
            .branch(
                teloxide::handler![State::AddBookTitle]
                    .endpoint(add_book_title_handler)
            )
            .branch(
                teloxide::handler![State::AddBookType { title }]
                    .endpoint(add_book_type_handler)
            )
            .branch(
                teloxide::handler![State::AddBookDescription { title, book_type }]
                    .endpoint(add_book_description_handler)
            )
            .branch(
                teloxide::handler![State::AddChapterId { book_id }]
                    .endpoint(add_chapter_id_handler)
            )
            .branch(
                teloxide::handler![State::AddChapterLink { book_id, chapter_id }]
                    .endpoint(add_chapter_link_handler)
            )
        )
        .branch(
            Update::filter_callback_query()
                .enter_dialogue::<CallbackQuery, InMemStorage<State>, State>()
                .branch(
                    teloxide::handler![State::AddBookTitle]
                        .endpoint(add_book_title_handler)
                )
                .endpoint(callback_handler)
        );

    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}

async fn up_database() {
    let url = dotenv::var("DATABASE_URL").unwrap();
    let db = sea_orm::Database::connect(url).await.unwrap();
    migrations::create_tables(&db).await;
}