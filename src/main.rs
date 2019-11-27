use tgslib::parse::available_types;
use std::fs::File;
use select::document::Document;
use tgslib::transform::types;

fn main() {
    const URL: &'static str = "https://core.telegram.org/bots/api";

    //let resp = reqwest::blocking::get(URL).unwrap();
    let file = File::open("./Telegram Bot API.html").unwrap();
    let doc = Document::from_read(file).unwrap();

    dbg!(tgslib::parse::recent_changes(&doc));
    //dbg!(types(parse_available_types(doc)));
}
