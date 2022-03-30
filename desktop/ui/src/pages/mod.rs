mod word;
mod sentence;
mod favorite;
mod setting;

pub use word::Word;
pub use sentence::Sentence;
pub use favorite::Favorite;
pub use setting::Setting;

use yew::prelude::*;

#[derive(Debug, PartialEq)]
pub enum PageType {
    Word,
    Sentence,
    Favorite,
    Setting,
}

impl PageType {
    pub fn html(&self) -> Html {
        match self {
            PageType::Word => html! {
                <Word />
            },
            PageType::Sentence => html! {
                <Sentence />
            },
            PageType::Favorite => html! {
                <Favorite />
            },
            PageType::Setting => html! {
                <Setting />
            },
        }
    }
}
