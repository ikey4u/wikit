mod ffi;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

enum AppMsg {
    SendHello,
    ReceivedMsg(String),
}

struct App {
    name: String,
    msg: String,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            name: "wikit".into(),
            msg: String::new(),
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <button class="button" onclick={ ctx.link().callback(|_| AppMsg::SendHello) }>{ "Test FFI" }</button>
                { self.msg.clone() }
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
