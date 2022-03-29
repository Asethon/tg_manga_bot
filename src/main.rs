use std::error::Error;
use teloxide::{
    payloads::SendMessageSetters,
    prelude2::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup,
    },
    utils::command::BotCommand,
};

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
    let mut row = vec![];
    match manga_id {
        Some(id) => {
            let mut chapters: Vec<Chapter> = vec![];
            let chapter1 = Chapter::new(1, 1, "1",
                                        "https://t.me/shrimp_from_the_island_bot/2");
            let chapter2 = Chapter::new(2, 2, "2",
                                        "https://t.me/shrimp_from_the_island_bot/6");
            chapters.push(chapter1);
            chapters.push(chapter2);
            row = chapters
                .into_iter()
                .filter(|chapter| chapter.manga_id == id)
                .map(|chapter| {
                    InlineKeyboardButton::callback(
                        "Глава ".to_owned() + chapter.chapter_id,
                        chapter.link.to_owned(),
                    )
                })
                .collect();
        }
        None => {
            let mut manga_list: Vec<Manga> = vec![];
            let manga = Manga::new(1, "Пик боевых искусств");
            manga_list.push(manga);
            row = manga_list
                .into_iter()
                .map(|manga| InlineKeyboardButton::callback(
                    manga.title.to_owned(),
                    "/manga?".to_owned() + &manga.id.to_string())
                )
                .collect();
        }
    }

    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    keyboard.push(row);

    InlineKeyboardMarkup::new(keyboard)
}

struct Manga<'a> {
    id: i32,
    title: &'a str,
}

impl<'a> Manga<'a> {
    fn new(id: i32, title: &'a str) -> Manga<'a> {
        Manga { id, title }
    }
}

struct Chapter<'b> {
    id: i32,
    manga_id: i32,
    chapter_id: &'b str,
    link: &'b str,
}

impl<'b> Chapter<'b> {
    fn new(id: i32, manga_id: i32, chapter_id: &'b str, link: &'b str) -> Chapter<'b> {
        Chapter { id, manga_id, chapter_id, link }
    }
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
    if let Some(command) = q.data {
        match q.message {
            Some(Message { id, chat, .. }) => {
                let split: Vec<&str> = command.split('?').collect();
                let count = split.into_iter().count();
                let text = format!("link: {}, {}", split[0], count);
                bot.send_message(chat.id, text).await?;
            }
            None => ()
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting bot...");

    let bot = Bot::from_env().auto_send();

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler).build().setup_ctrlc_handler().dispatch().await;

    log::info!("Closing bot... Goodbye!");

    Ok(())
}