mod ffi;
mod pages;
mod dom;

use pages::PageType;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use gloo::utils::document;
use web_sys::{KeyboardEvent, EventTarget};

enum AppMsg {
    SendHello,
    ReceivedMsg(String),
    GotoWordPage,
    GotoSentencePage,
    GotoFavoritePage,
    GotoSettingPage,
    OnSearchTextChange(String),
}

enum Fileds {
    SearchInput,
}

impl Fileds {
    pub fn value(&self) -> String {
        match self {
            SearchInput => {
                dom::get_input_value("search")
            }
        }
    }
}

struct App {
    name: String,
    msg: String,
    // current page
    page: PageType,
    // query keyword
    input: String,
    // list of hits of the query keyword
    fuzzy_list: Vec<String>,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            name: "wikit".into(),
            msg: String::new(),
            page: PageType::Word,
            input: String::new(),
            fuzzy_list: vec![],
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
            AppMsg::OnSearchTextChange(text) => {
                if text.len() > 0 {
                    self.fuzzy_list = (1..100).map(|i| format!("{text}")).collect();
                } else {
                    self.fuzzy_list = vec![];
                }
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
        html! {
            <div>
                <nav class="navbar is-size-7 wikit-menu">
                    <div class="navbar-menu">
                        <div class="navbar-start">
                            <a class="navbar-item is-active has-text-centered" onclick={ ctx.link().callback(|_| AppMsg::GotoWordPage) }>
                                <div>
                                  <p>
                                    <span class="icon is-small is-centered">
                                      <i class="bi bi-braces"></i>
                                    </span>
                                  </p>
                                  <p>{ "Word" }</p>
                                </div>
                            </a>
                            <a class="navbar-item has-text-centered" onclick={ ctx.link().callback(|_| AppMsg::GotoSentencePage) }>
                                <div>
                                  <p>
                                    <span class="icon is-small is-centered">
                                      <i class="bi bi-braces-asterisk"></i>
                                    </span>
                                  </p>
                                  <p>{ "Sentence" }</p>
                                </div>
                            </a>
                            <a class="navbar-item has-text-centered" onclick={ ctx.link().callback(|_| AppMsg::GotoFavoritePage) }>
                                <div>
                                  <p>
                                    <span class="icon is-small is-centered">
                                      <i class="bi bi-heart"></i>
                                    </span>
                                  </p>
                                  <p>{ "Favorite" }</p>
                                </div>
                            </a>
                            if self.page != PageType::Setting {
                                <div class="navbar-item">
                                    <div class="field">
                                      <p class="control has-icons-left">
                                        <input
                                            class="input is-rounded is-small"
                                            autocomplete="none" autocorrect="off" autocapitalize="none"
                                            type="text"
                                            id="search"
                                            onkeyup={
                                                ctx.link().callback(|_| {
                                                    AppMsg::OnSearchTextChange(Fileds::SearchInput.value())
                                                })
                                            }
                                        />
                                        <span class="icon is-small is-left">
                                          <i class="bi bi-search"></i>
                                        </span>
                                      </p>
                                    </div>
                                </div>
                            }
                        </div>
                        <div class="navbar-end">
                            <a class="navbar-item has-text-centered" onclick={ ctx.link().callback(|_| AppMsg::GotoSettingPage) }>
                                <div>
                                  <p>
                                    <span class="icon is-small is-centered">
                                      <i class="bi bi-gear"></i>
                                    </span>
                                  </p>
                                  <p>{ "Setting" }</p>
                                </div>
                            </a>
                        </div>
                    </div>
                </nav>
                <div class="columns is-mobile">
                    if self.fuzzy_list.len() > 0 {
                        <div class="column is-one-fifth wikit-list">
                          {
                              self.fuzzy_list.clone().into_iter().map(|item| {
                                  html!{
                                    <div>
                                        { item }
                                    </div>
                                  }
                              }).collect::<Html>()
                          }
                        </div>
                    }
                    <div class="column with-wikit-body-height">
                        <div class="section">
                        </div>
                        <div class="section">
                            { self.page.html() }
                        </div>
                    </div>
                </div>
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    let yewapp = document().get_element_by_id("yewapp").expect("failed to get yewapp element");
    yew::Renderer::<App>::with_root(yewapp).render();
}
