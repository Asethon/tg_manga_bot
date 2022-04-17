use entity::book::BookType;

fn main() {
    let mut str = String::new();
    match BookType::try_from(1).unwrap() {
        BookType::Manga => str = String::from("manga"),
        BookType::Ranobe => str = String::from("ranobe"),
    }
    println!("{:?}", str);
}