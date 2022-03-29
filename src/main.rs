use std::collections::HashMap;
use std::error::Error;
use teloxide::{
    payloads::SendMessageSetters,
    prelude2::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup,
    },
    utils::command::BotCommand,
};
use teloxide::types::ParseMode::MarkdownV2;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Start")]
    Start,
    #[command(description = "ping-pong")]
    Ping,
}

fn make_keyboard() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let catalog = [
        "Пик Боевых Искусств"
    ];

    let row = catalog
        .iter()
        .map(|&version| InlineKeyboardButton::callback(version.to_owned(), "/pick".to_owned()))
        .collect();

    keyboard.push(row);

    InlineKeyboardMarkup::new(keyboard)
}

fn make_keyboard2() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
    let mut chapters: HashMap<&str, &str> = HashMap::new();

    chapters.insert("https://t.me/shrimp_from_the_island_bot/2", "Глава 1");
    chapters.insert("https://t.me/shrimp_from_the_island_bot/6", "Глава 2");

    let row = chapters
        .into_iter()
        .map(|version| InlineKeyboardButton::callback(version.1.to_owned(), version.0.to_owned()))
        .collect();

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
                // Create a list of buttons and send them.
                let keyboard = make_keyboard();
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
                if command == "/pick" {
                    bot.edit_message_text(chat.id, id, "Выберите главу:")
                        .reply_markup(make_keyboard2()).await?;
                } else {
                    let mut chapters: HashMap<String, &str> = HashMap::new();

                    chapters.insert(String::from("https://t.me/shrimp_from_the_island_bot/2"), "Глава 1");
                    chapters.insert(String::from("https://t.me/shrimp_from_the_island_bot/6"), "Глава 2");
                    let chapter = *chapters.get(&command).unwrap();
                    let text = format!("[{}]({})", chapter, command);
                    bot.edit_message_text(chat.id, id, text).parse_mode(MarkdownV2).await?;
                }
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