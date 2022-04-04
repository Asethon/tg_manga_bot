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

use crate::database::{
    chapter::ChapterRepository,
    database::DatabaseConnection,
    manga::MangaRepository,
};

mod database;

type MangaDialogue = Dialogue<State, InMemStorage<State>>;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Start")]
    Start,
    #[command(description = "Main menu")]
    Menu,

    #[command(description = "Add manga")]
    MangaAdd,
    #[command(description = "Add chapter")]
    ChapterAdd { id: i32 },

    #[command(description = "Delete manga")]
    MangaDelete(i32),

    #[command(description = "ping-pong")]
    Ping,
}

async fn make_keyboard(manga_id: Option<i32>) -> InlineKeyboardMarkup {
    let row;
    let client = DatabaseConnection::client().await.unwrap();
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
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
            keyboard.push(vec![InlineKeyboardButton::callback(
                "Добавить главу".to_owned(),
                "/chapter_add?".to_owned() + &*id.to_string(),
            )]);
        }
        None => {
            row = MangaRepository::init(client).list().await.unwrap()
                .into_iter()
                .map(|manga| InlineKeyboardButton::callback(
                    manga.title.to_owned() + "[" + &manga.id.unwrap().to_string() + "]",
                    "/manga?".to_owned() + &manga.id.unwrap().to_string())
                )
                .collect();
            keyboard.push(vec![InlineKeyboardButton::callback(
                "Добавить мангу".to_owned(),
                "/mangaadd".to_owned(),
            )]);
        }
    }

    keyboard.push(row);

    InlineKeyboardMarkup::new(keyboard)
}

/// Parse the text wrote on Telegram and check if that text is a valid command
/// or not, then match the command. If the command is `/start` it writes a
/// markup with the `InlineKeyboardMarkup`.
async fn message_handler(
    bot: AutoSend<Bot>,
    m: Message,
    manga_dialogue: MangaDialogue,
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
            Ok(Command::MangaAdd) => {
                bot.send_message(m.chat.id, "Add manga...").await?;
                manga_dialogue.update(State::AddMangaTitle).await?;
            }
            Ok(Command::MangaDelete(id)) => {
                let client = DatabaseConnection::client().await?;
                MangaRepository::init(client).delete(id).await?;
                bot.send_message(m.chat.id, "Manga deleted").await?;
            }
            Ok(Command::ChapterAdd { id }) => {
                bot.send_message(m.chat.id, "Add chapter...").await?;
                manga_dialogue.update(State::InsertChapterId { id }).await?;
            }
            Err(_) => {
                let text = format!("Что-то пошло не так... {}", text);
                bot.send_message(m.chat.id, text).await?;
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
#[handler_out(anyhow::Result < () >)]
pub enum State {
    #[handler(message_handler)]
    Start,

    ///Manga
    #[handler(add_manga_title_handler)]
    AddMangaTitle,
    #[handler(add_manga_description_handler)]
    Description { title: String },

    ///Chapter
    #[handler(chapter_id_handler)]
    InsertChapterId { id: i32 },
    #[handler(chapter_link_handler)]
    InsertChapterLink { id: i32, chapter_id: String },
}

async fn add_manga_title_handler(
    bot: AutoSend<Bot>,
    m: Message,
    dialogue: MangaDialogue,
) -> anyhow::Result<()> {
    match m.text() {
        Some(text) => {
            bot.send_message(m.chat.id, "Send me description").await?;
            dialogue.update(State::Description { title: text.into() }).await?;
        }
        None => ()
    }
    Ok(())
}

async fn add_manga_description_handler(
    bot: AutoSend<Bot>,
    m: Message,
    dialogue: MangaDialogue,
    (title, ): (String, ),
) -> anyhow::Result<()> {
    match m.text() {
        Some(text) => {
            bot.send_message(m.chat.id, "Manga added").await?;
            let client = DatabaseConnection::client().await?;
            MangaRepository::init(client).new(1, title, text.to_string(), "image".to_string()).push().await?;
            dialogue.update(State::Start).await?;
        }
        None => ()
    }
    Ok(())
}

impl Default for State {
    fn default() -> Self {
        Self::Start
    }
}

async fn chapter_id_handler(
    bot: AutoSend<Bot>,
    m: Message,
    dialogue: MangaDialogue,
    (id, ): (i32, ),
) -> anyhow::Result<()> {
    match m.text() {
        Some(chapter_id) => {
            bot.send_message(m.chat.id, "Send link:").await?;
            dialogue.update(State::InsertChapterLink { id, chapter_id: chapter_id.to_string() }).await?;
        }
        None => ()
    }
    Ok(())
}

async fn chapter_link_handler(
    bot: AutoSend<Bot>,
    m: Message,
    dialogue: MangaDialogue,
    (id, chapter_id): (i32, String),
) -> anyhow::Result<()> {
    match m.text() {
        Some(link) => {
            let client = DatabaseConnection::client().await?;
            let manga = MangaRepository::init(client).get_by_id(id).await?;
            let text = format!("{} [Глава {}]({})", manga.title, chapter_id, link);
            bot.send_message(m.chat.id, text).parse_mode(MarkdownV2).await?;
            dialogue.update(State::Start).await?;
        }
        None => ()
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    database::init::create_tables().await;
    pretty_env_logger::init();
    log::info!("Starting bot...");
    dotenv::dotenv().ok();

    let bot = Bot::from_env().auto_send();

    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(
                    dptree::entry()
                        .enter_dialogue::<Message, InMemStorage<State>, State>()
                        .dispatch_by::<State>()
                )
                .endpoint(message_handler)
        )
        .branch(
            Update::filter_callback_query().endpoint(callback_handler)
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .build().setup_ctrlc_handler().dispatch().await;

    log::info!("Closing bot... Goodbye!");

    Ok(())
}