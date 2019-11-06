use tgslib::step1::parse_available_types;
use std::fs::File;
use select::document::Document;
use tgslib::step2::types;

fn main() {
    const URL: &'static str = "https://core.telegram.org/bots/api";

    //let resp = reqwest::blocking::get(URL).unwrap();
    let file = File::open("./Telegram Bot API.html").unwrap();
    let doc = Document::from_read(file).unwrap();

    dbg!(types(parse_available_types(doc)));
}
