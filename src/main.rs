use std::error::Error;
use std::process::exit;
use teloxide::{
    payloads::SendMessageSetters,
    prelude2::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, ParseMode::MarkdownV2
    },
    utils::command::BotCommand,
};

use crate::database::chapter::ChapterRepository;
use crate::database::manga::MangaRepository;

mod database;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Start")]
    Start,
    #[command(description = "Main menu")]
    Menu,
    #[command(description = "ping-pong")]
    Ping,
}

fn make_keyboard(manga_id: Option<i32>) -> InlineKeyboardMarkup {
    let row;
    match manga_id {
        Some(id) => {
            row = ChapterRepository::list_by_manga_id(&mut Default::default(), id).unwrap()
                .into_iter()
                .map(|chapter| {
                    InlineKeyboardButton::callback(
                        "Глава ".to_owned() + &*chapter.chapter_id,
                        "/chapter?".to_owned() + &chapter.id.unwrap().to_string(),
                    )
                })
                .collect();
        }
        None => {
            row = MangaRepository::list(&mut Default::default()).unwrap()
                .into_iter()
                .map(|manga| InlineKeyboardButton::callback(
                    manga.title.to_owned(),
                    "/manga?".to_owned() + &manga.id.unwrap().to_string())
                )
                .collect();
        }
    }

    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    keyboard.push(row);

    InlineKeyboardMarkup::new(keyboard)
}

/// Parse the text wrote on Telegram and check if that text is a valid command
/// or not, then match the command. If the command is `/start` it writes a
/// markup with the `InlineKeyboardMarkup`.
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
                bot.send_message(m.chat.id, "Hi, send me /menu").await?;
            }
            Ok(Command::Menu) => {
                let keyboard = make_keyboard(None);
                bot.send_message(m.chat.id, "Каталог:").reply_markup(keyboard).await?;
            }
            Ok(Command::Ping) => {
                bot.send_message(m.chat.id, "pong").await?;
            }
            Err(_) => {
                bot.send_message(m.chat.id, "Что-то пошло не так...").await?;
            }
        };
    }

    Ok(())
}

/// When it receives a callback from a button it edits the message with all
/// those buttons writing a text with the selected Debian version.
///
/// **IMPORTANT**: do not send privacy-sensitive data this way!!!
/// Anyone can read data stored in the callback button.
async fn callback_handler(
    q: CallbackQuery,
    bot: AutoSend<Bot>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(link) = q.data {
        match q.message {
            Some(Message { id, chat, .. }) => {
                let split: Vec<&str> = link.split('?').collect();
                let link_id: i32 = split[1].parse::<i32>().unwrap();
                match split[0] {
                    "/manga" => {
                        let keyboard = make_keyboard(Some(link_id));
                        bot.edit_message_text(chat.id, id, "Главы:").reply_markup(keyboard).await?;
                    }
                    "/chapter" => {
                        let chapter= ChapterRepository::get_by_id(&mut Default::default(), link_id)?;
                        let link = format!("[Глава {}]({})", chapter.id.unwrap(), chapter.link);
                        let keyboard = make_keyboard(Some(chapter.manga_id));
                        bot.edit_message_text(chat.id, id, link).reply_markup(keyboard).parse_mode(MarkdownV2).await?;
                    }
                    _ => {}
                }
            }
            None => ()
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let migrate = std::env::args().nth(1);
    match migrate {
        Some(_) => {
            database::init::create_tables();
            exit(0);
        }
        None => {}
    }
    pretty_env_logger::init();
    log::info!("Starting bot...");
    dotenv::dotenv().ok();

    let bot = Bot::from_env().auto_send();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler).build().setup_ctrlc_handler().dispatch().await;

    log::info!("Closing bot... Goodbye!");

    Ok(())
}