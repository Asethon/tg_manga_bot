use teloxide::types::{KeyboardButton, KeyboardMarkup, ReplyMarkup};

pub fn make_keyboard(keyboard: Vec<Vec<String>>) -> ReplyMarkup {
    let keyboard: Vec<Vec<KeyboardButton>> = keyboard.iter()
        .map(|row| {
            row.iter()
                .map(|label| KeyboardButton::new(label))
                .collect()
        }).collect();
    let markup = KeyboardButton::new(keyboard)
        .resize_keyboard(true);
    ReplyMarkup::Keyboard(markup)
}

pub fn cancel_markup() -> ReplyMarkup {
   kb_markup(vec![vec![String::from("/")]])
}