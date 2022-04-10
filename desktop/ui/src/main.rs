mod ffi;
mod pages;
mod dom;

use pages::PageType;
use ffi::FFIResult;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use gloo::utils::document;
use web_sys::{KeyboardEvent, EventTarget};
use wasm_bindgen::JsValue;

enum AppMsg {
    SendHello,
    ReceivedMsg(String),
    GotoWordPage,
    GotoSentencePage,
    GotoFavoritePage,
    GotoSettingPage,
    OnSearchTextChange(String),
    GoToEditorPage,
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
    current_dictionary: String,
    dictionary_list: Vec<String>,
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
            current_dictionary: "".into(),
            dictionary_list: vec![],
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
                if text.trim().len() > 0 {
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
            AppMsg::GoToEditorPage => {
                spawn_local(async {
                    async fn start_preview_server() -> FFIResult<()> {
                        let sourcedir = ffi::open().await?
                            .as_string()
                            .ok_or(JsValue::from_str("failed to get source dir"))?;
                        ffi::start_preview_server(sourcedir).await?;
                        Ok(())
                    }
                    if let Err(e) = start_preview_server().await {
                        log::error!("failed to start preview server: {:?}", e);
                        return;
                    }
                    log::info!("server is started");
                });
                // TODO(2022-04-10): toggle to preview page only when live server is up
                self.page = PageType::Editor;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let fouces_menu_class = |typ: PageType| {
            if self.page == typ {
                "navbar-item is-active has-text-centered has-text-link"
            } else {
                "navbar-item is-active has-text-centered"
            }
        };
        html! {
            <div>
                <nav class="navbar is-size-7 wikit-menu">
                    <div class="navbar-menu">
                        <div class="navbar-start">
                            <a class={ fouces_menu_class(PageType::Word) } onclick={ ctx.link().callback(|_| AppMsg::GotoWordPage) }>
                                <div>
                                  <p>
                                    <span class="icon is-small is-centered">
                                      <i class="bi bi-braces"></i>
                                    </span>
                                  </p>
                                  <p>{ "Word" }</p>
                                </div>
                            </a>
                            <div class="navbar-item has-dropdown is-hoverable">
                                <a class="navbar-link">
                                    { "Advanced" }
                                </a>
                                <div class="navbar-dropdown">
                                    <a class="navbar-item" onclick={ ctx.link().callback(|_| AppMsg::GoToEditorPage) }>
                                        { "Editor Preview" }
                                    </a>
                                    <a class="navbar-item is-hidden-mobile">
                                        { "Converter" }
                                    </a>
                                </div>
                            </div>
                            <a class="navbar-item has-text-centered is-hidden-mobile" onclick={ ctx.link().callback(|_| AppMsg::GotoSentencePage) }>
                                <div>
                                  <p>
                                    <span class="icon is-small is-centered">
                                      <i class="bi bi-braces-asterisk"></i>
                                    </span>
                                  </p>
                                  <p>{ "Sentence" }</p>
                                </div>
                            </a>
                            <a class="navbar-item has-text-centered is-hidden-mobile" onclick={ ctx.link().callback(|_| AppMsg::GotoFavoritePage) }>
                                <div>
                                  <p>
                                    <span class="icon is-small is-centered">
                                      <i class="bi bi-heart"></i>
                                    </span>
                                  </p>
                                  <p>{ "Favorite" }</p>
                                </div>
                            </a>
                            if self.page == PageType::Word {
                                <div class="navbar-item">
                                    <div class="field has-addons">
                                      <p class="control">
                                        <span class="select is-rounded is-small">
                                          <select>
                                            <option>{ "Oxford" }</option>
                                            <option>{ "剑桥词典" }</option>
                                            <option>{ "韦氏词典" }</option>
                                            <option>{ "添加词典" }</option>
                                          </select>
                                        </span>
                                      </p>
                                          <p class="control has-icons-left is-expanded">
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
                            if self.page == PageType::Editor {
                                <div class="navbar-item">
                                    <div class="buttons">
                                        <a class="button is-primary is-small">
                                            <strong>{ "Start" }</strong>
                                        </a>
                                        <a class="button is-black is-small">
                                            <strong>{ "Stop" }</strong>
                                        </a>
                                    </div>
                                </div>
                            }
                        </div>
                        <div class="navbar-end">
                            <a class={ fouces_menu_class(PageType::Setting) } onclick={ ctx.link().callback(|_| AppMsg::GotoSettingPage) }>
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
                    if self.fuzzy_list.len() > 0  && self.page == PageType::Word {
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
                        <div class="container">
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
