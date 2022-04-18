use std::error::Error;
use teloxide::Bot;
use teloxide::{
    payloads::SendMessageSetters,
    prelude2::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText,
    },
    utils::command::BotCommand,
    dispatching2::dialogue::InMemStorage,
    macros::DialogueState
};

mod db;
use db::migrations;
use crate::db::{BookRepository, ChapterRepository};

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Start")]
    Start,
}

async fn make_keyboard() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let url = dotenv::var("DATABASE_URL").unwrap();
    let db = sea_orm::Database::connect(url).await.unwrap();
    let book_repository = BookRepository { db: db.clone() };
    let books = book_repository.find_by_filter().await;

    for book in books.chunks(1) {
        let row = book
            .iter()
            .map(|book| InlineKeyboardButton::callback(book.title.clone(), book.id.to_string()))
            .collect();
        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

async fn message_handler(
    m: Message,
    bot: AutoSend<Bot>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = m.text() {
        match BotCommand::parse(text, "buttons") {
            Ok(Command::Help) => {
                // Just send the description of all commands.
                bot.send_message(m.chat.id, Command::descriptions()).await?;
            }
            Ok(Command::Start) => {
                // Create a list of buttons and send them.
                let keyboard = make_keyboard().await;
                bot.send_message(m.chat.id, "Books list:").reply_markup(keyboard).await?;
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
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(version) = q.data {
        let text = format!("You chose: {}", version);

        match q.message {
            Some(Message { id, chat, .. }) => {
                bot.edit_message_text(chat.id, id, text).await?;
            }
            None => {
                if let Some(id) = q.inline_message_id {
                    bot.edit_message_text_inline(id, text).await?;
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let bot = Bot::from_env().auto_send();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
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