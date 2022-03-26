mod word;
mod sentence;
mod favorite;
mod setting;

pub use word::Word;
pub use sentence::Sentence;
pub use favorite::Favorite;
pub use setting::Setting;

#[derive(Debug)]
pub enum PageType {
    Word,
    Sentence,
    Favorite,
    Setting,
}
