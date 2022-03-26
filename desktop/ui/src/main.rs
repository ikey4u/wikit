mod ffi;
mod pages;
mod dom;

use pages::PageType;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use gloo::utils::document;

enum AppMsg {
    SendHello,
    ReceivedMsg(String),
    GotoWordPage,
    GotoSentencePage,
    GotoFavoritePage,
    GotoSettingPage,
}

struct App {
    name: String,
    msg: String,
    page: PageType,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            name: "wikit".into(),
            msg: String::new(),
            page: PageType::Word,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMsg::SendHello => {
                let link = ctx.link().clone();
                let name = self.name.clone();
                spawn_local(async move {
                    match ffi::ffi_hello(name).await {
                        Ok(msg) => {
                            let msg = msg.as_string().expect("ffi_hello should return string".into());
                            link.send_message(AppMsg::ReceivedMsg(msg));
                        }
                        Err(e) => {
                            log::info!("failed to get message: {:?}", e);
                        }
                    }
                });
                true
            }
            AppMsg::ReceivedMsg(msg) => {
                self.msg = msg;
                true
            }
            AppMsg::GotoWordPage => {
                self.page = PageType::Word;
                true
            }
            AppMsg::GotoSentencePage => {
                self.page = PageType::Sentence;
                true
            }
            AppMsg::GotoFavoritePage => {
                self.page = PageType::Favorite;
                true
            }
            AppMsg::GotoSettingPage => {
                self.page = PageType::Setting;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let page = match self.page {
            PageType::Word => html! {
                <pages::Word />
            },
            PageType::Sentence => html! {
                <pages::Sentence />
            },
            PageType::Favorite => html! {
                <pages::Favorite />
            },
            PageType::Setting => html! {
                <pages::Setting />
            },
        };
        html! {
            <div>
                <nav class="navbar is-size-7">
                    <div class="navbar-menu is-active">
                        <div class="navbar-start">
                            <a class="navbar-item" onclick={ ctx.link().callback(|_| AppMsg::GotoWordPage) }>{ "Word" }</a>
                            <a class="navbar-item" onclick={ ctx.link().callback(|_| AppMsg::GotoSentencePage) }>{ "Sentence" }</a>
                            <a class="navbar-item" onclick={ ctx.link().callback(|_| AppMsg::GotoFavoritePage) }>{ "Favorite" }</a>
                        </div>
                        <div class="navbar-end">
                            <a class="navbar-item" onclick={ ctx.link().callback(|_| AppMsg::GotoSettingPage) }>{ "Setting" }</a>
                        </div>
                    </div>
                </nav>
                { page }
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    let yewapp = document().get_element_by_id("yewapp").expect("failed to get yewapp element");
    yew::Renderer::<App>::with_root(yewapp).render();
}
