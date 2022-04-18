use sea_orm::DatabaseConnection;
use teloxide::Bot;
use teloxide::{
    payloads::SendMessageSetters,
    prelude2::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup,
    },
    utils::command::BotCommand,
    dispatching2::dialogue::InMemStorage,
    macros::DialogueState,
};
use teloxide::types::ParseMode::MarkdownV2;

pub mod db;
pub mod domain;

use db::migrations;
use crate::db::{BookRepository, ChapterRepository};
use crate::domain::books::book::BookType;


#[derive(BotCommand)]
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

async fn make_keyboard(book_id: Option<i32>) -> InlineKeyboardMarkup {
    let mut row = vec![];
    let url = dotenv::var("DATABASE_URL").unwrap();
    let db = sea_orm::Database::connect(url).await.unwrap();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    match book_id {
        Some(id) => {
            let repository = ChapterRepository { db: db.clone() };
            let chapters = repository.find_by_book_id(id).await;
            row = chapters
                .into_iter()
                .map(|chapter| {
                    let ch = format!("Глава: {}", chapter.chapter_id);
                    let link = format!("/chapter?{}", chapter.chapter_id);
                    InlineKeyboardButton::callback(ch, link)
                })
                .collect();
        }
        None => {
            let repository = BookRepository { db: db.clone() };
            let books = repository.find_by_filter().await;
            row = books
                .into_iter()
                .map(|book| {
                    let link = format!("/book?{}", book.id);
                    InlineKeyboardButton::callback(
                        book.title.clone(),
                        link,
                    )
                })
                .collect();
        }
    }

    keyboard.push(row);
    InlineKeyboardMarkup::new(keyboard)
}

async fn message_handler(
    m: Message,
    bot: AutoSend<Bot>,
    dialogue: BookDialogue,
) -> anyhow::Result<()> {
    if let Some(text) = m.text() {
        match BotCommand::parse(text, "buttons") {
            Ok(Command::Help) => {
                // Just send the description of all commands.
                bot.send_message(m.chat.id, Command::descriptions()).await?;
            }
            Ok(Command::Start) => {
                bot.send_message(m.chat.id, "Hi, send me /menu").await?;
            }

            Ok(Command::Menu) => {
                let keyboard = make_keyboard(None).await;
                bot.send_message(m.chat.id, "Каталог:").reply_markup(keyboard).await?;
            }

            Ok(Command::BookAdd) => {
                bot.send_message(m.chat.id, "Введите название произведения: ").await?;
                dialogue.update(State::AddBookTitle).await?;
            }

            Ok(Command::ChapterAdd { .. }) => {
                /*bot.send_message(m.chat.id, "Введите название произведения: ").await?;
                dialogue.update(State::AddBookTitle).await?;*/
            }

            Ok(Command::Ping) => {
                bot.send_message(m.chat.id, "pong").await?;
            }

            Err(_) => {
                bot.send_message(m.chat.id, "Command not found!").await?;
            }
        }
    }

    Ok(())
}

async fn callback_handler(
    q: CallbackQuery,
    bot: AutoSend<Bot>,
) -> anyhow::Result<()> {
    if let Some(link) = q.data {
        match q.message {
            Some(Message { chat, .. }) => {
                let split: Vec<&str> = link.split('?').collect();
                let link_id: i32 = split[1].parse::<i32>().unwrap();
                match split[0] {
                    "/book" => {
                        let keyboard = make_keyboard(Some(link_id)).await;
                        bot.send_message(chat.id, "Главы:").reply_markup(keyboard).await?;
                    }
                    "/chapter" => {
                        let url = dotenv::var("DATABASE_URL").unwrap();
                        let db = sea_orm::Database::connect(url).await.unwrap();
                        let repository = ChapterRepository { db };
                        let chapter = repository.find_by_id(link_id).await;
                        let keyboard = make_keyboard(Some(chapter.book_id)).await;
                        let link = format!("[Глава {}]({})", chapter.id, chapter.link);
                        bot.send_message(chat.id, link).reply_markup(keyboard).parse_mode(MarkdownV2).await?;
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

#[derive(DialogueState, Clone)]
#[handler_out(anyhow::Result < () >)]
pub enum State {
    #[handler(message_handler)]
    Start,

    ///Book
    #[handler(add_book_title_handler)]
    AddBookTitle,
    #[handler(add_book_type_handler)]
    AddBookType { title: String },
    #[handler(add_book_description_handler)]
    AddBookDescription { title: String, book_type: String },

    /*///Chapter
    #[handler(message_handler)]
    AddChapterId { book_id: i32 },
    #[handler(message_handler)]
    AddChapterLink { book_id: i32, chapter_id: String },*/
}

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
        None => (),
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
    (title, ): (String, )
) -> anyhow::Result<()> {
    match m.text() {
        None => (),
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
    (title, book_type): (String, String )
) -> anyhow::Result<()> {
    match m.text() {
        None => (),
        Some(description) => {
            let db = get_db().await;
            let repository = BookRepository { db };
            repository.insert(
                BookType::try_from(&*book_type).unwrap(),
                title,
                description.into(),
                "None".into()
            ).await;
            bot.send_message(m.chat.id, "Произведение добавлено").await?;
            dialogue.update(State::Start).await?;
        }
    }
    Ok(())
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
                    .dispatch_by::<State>(),
        )
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
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