mod ffi;
mod pages;
mod dom;
mod util;

use std::time::Duration;

use pages::PageType;
use ffi::FFIResult;

use yew::prelude::*;
use gloo::utils::document;
use gloo::timers;
use gloo::net::http;
use web_sys::{KeyboardEvent, EventTarget};
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen::JsValue;

enum AppMsg {
    SendHello,
    ReceivedMsg(String),
    GotoWordPage,
    GotoSentencePage,
    GotoFavoritePage,
    GotoSettingPage,
    GoToEditorPage,
    StartPreviewer,
}

struct App {
    name: String,
    msg: String,
    // current page
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
                            log::error!("failed to get message: {:?}", e);
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
            AppMsg::GoToEditorPage => {
                self.page = PageType::Editor;
                true
            }
            AppMsg::StartPreviewer => {
                // start preview server
                spawn_local(async {
                    async fn start_preview_server() -> FFIResult<()> {
                        if let Some(started) = ffi::is_preview_server_up().await?.as_bool() {
                            if started {
                                return Ok(())
                            }
                        }
                        let sourcedir = ffi::open().await?
                            .as_string()
                            .ok_or(JsValue::from_str("failed to get source dir"))?;
                        ffi::start_preview_server(sourcedir).await?;
                        Ok(())
                    }
                    if let Err(e) = start_preview_server().await {
                        log::error!("failed to start preview server: {:?}", e);
                    }
                });

                // check if preview server is up, if the server is up, then we update the page
                let link = ctx.link().clone();
                spawn_local(async move {
                    loop {
                        if let Ok(resp) = http::Request::get("http://127.0.0.1:8088").send().await {
                            if resp.status() == 200 {
                                link.send_message(AppMsg::GoToEditorPage);
                            }
                            break;
                        }
                        timers::future::sleep(Duration::from_secs(1)).await;
                    }
                });
                false
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
                            <a class="navbar-item has-text-centered is-hidden" onclick={ ctx.link().callback(|_| AppMsg::GotoSentencePage) }>
                                <div>
                                  <p>
                                    <span class="icon is-small is-centered">
                                      <i class="bi bi-braces-asterisk"></i>
                                    </span>
                                  </p>
                                  <p>{ "Sentence" }</p>
                                </div>
                            </a>
                            <a class="navbar-item has-text-centered is-hidden" onclick={ ctx.link().callback(|_| AppMsg::GotoFavoritePage) }>
                                <div>
                                  <p>
                                    <span class="icon is-small is-centered">
                                      <i class="bi bi-heart"></i>
                                    </span>
                                  </p>
                                  <p>{ "Favorite" }</p>
                                </div>
                            </a>
                            <div class="navbar-item has-dropdown is-hoverable">
                                <a class="navbar-link">
                                    { "Advanced" }
                                </a>
                                <div class="navbar-dropdown">
                                    <a class="navbar-item" onclick={ ctx.link().callback(|_| AppMsg::StartPreviewer ) }>
                                        { "Live Editor" }
                                    </a>
                                    <a class="navbar-item is-hidden">
                                        { "Converter" }
                                    </a>
                                </div>
                            </div>
                        </div>
                        <div class="navbar-end is-hidden">
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
                <div class="columns mt-4">
                    <div class="column with-wikit-body-height">
                        <div class="container">
                            <div class="notification is-primary is-hidden">
                                {"notification ..."}
                            </div>
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
    yew::start_app_in_element::<App>(yewapp);
}
