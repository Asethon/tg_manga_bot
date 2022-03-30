use std::error::Error;
use teloxide::{
    payloads::SendMessageSetters,
    prelude2::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, ParseMode::MarkdownV2,
    },
    macros::DialogueState,
    utils::command::BotCommand,
};
use teloxide::dispatching2::dialogue::InMemStorage;

use crate::database::chapter::ChapterRepository;
use crate::database::database::DatabaseConnection;
use crate::database::manga::{Manga, MangaRepository};

mod database;

type MyDialogue = Dialogue<State, InMemStorage<State>>;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Start")]
    Start,
    #[command(description = "Main menu")]
    Menu,
    // #[command(description = "Add manga")]
    // AddManga,
    #[command(description = "ping-pong")]
    Ping,
}

async fn make_keyboard(manga_id: Option<i32>) -> InlineKeyboardMarkup {
    let row;
    let client = DatabaseConnection::client().await.unwrap();
    match manga_id {
        Some(id) => {
            row = ChapterRepository::init(client).await.list_by_manga_id(id).await.unwrap()
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
            let manga = vec![Manga { id: None, group_id: 0, title: "Title".to_string(), description: "Desc".to_string(), img: "image".to_string() }];
            row = manga
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
    bot: AutoSend<Bot>,
    m: Message,
    dialogue: MyDialogue,
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
) -> anyhow::Result<()> {
    if let Some(link) = q.data {
        match q.message {
            Some(Message { id, chat, .. }) => {
                let split: Vec<&str> = link.split('?').collect();
                let link_id: i32 = split[1].parse::<i32>().unwrap();
                match split[0] {
                    "/manga" => {
                        let keyboard = make_keyboard(Some(link_id)).await;
                        bot.edit_message_text(chat.id, id, "Главы:").reply_markup(keyboard).await?;
                    }
                    "/chapter" => {
                        let client = DatabaseConnection::client().await?;
                        let chapter = ChapterRepository::init(client).await.get_by_id(link_id).await?;
                        let link = format!("[Глава {}]({})", chapter.id.unwrap(), chapter.link);
                        let keyboard = make_keyboard(Some(chapter.manga_id)).await;
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
#[derive(DialogueState, Clone)]
#[handler_out(anyhow::Result<()>)]
pub enum State {
    #[handler(message_handler)]
    Start,
}

impl Default for State {
    fn default() -> Self {
        Self::Start
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    log::info!("Starting bot...");
    dotenv::dotenv().ok();

    let bot = Bot::from_env().auto_send();

    let handler = dptree::entry()
        .branch(Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .dispatch_by::<State>()
        )
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .build().setup_ctrlc_handler().dispatch().await;

    log::info!("Closing bot... Goodbye!");

    Ok(())
}