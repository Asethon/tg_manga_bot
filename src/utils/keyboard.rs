use teloxide::types::{KeyboardButton, KeyboardMarkup, ReplyMarkup};
use strum::{AsRefStr, EnumString,};

#[derive(AsRefStr, EnumString)]
pub enum Command {
    #[strum(to_string = "/")]
    Cancel
}

pub fn make_keyboard(keyboard: Vec<Vec<String>>) -> ReplyMarkup {
    let keyboard: Vec<Vec<KeyboardButton>> = keyboard.iter()
        .map(|row| {
            row.iter()
                .map(|label| KeyboardButton::new(label))
                .collect()
        }).collect();
    let markup = KeyboardMarkup::new(keyboard)
        .resize_keyboard(true);
    ReplyMarkup::Keyboard(markup)
}

pub fn cancel_markup() -> ReplyMarkup {
   make_keyboard(vec![vec![String::from("/ ")]])
}