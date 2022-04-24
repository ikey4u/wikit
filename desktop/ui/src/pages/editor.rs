use wasm_bindgen::JsCast;
use yew::prelude::*;
use gloo::{net::websocket::{Message, futures::WebSocket}, utils::document};
use wasm_bindgen_futures::spawn_local;
use futures::{SinkExt, StreamExt};
use web_sys::HtmlIFrameElement;

pub struct EditorMsg {
}

pub struct Editor;

impl Component for Editor {
    type Message = EditorMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onload = Callback::from(move |e: Event| {
            let ws = WebSocket::open("ws://127.0.0.1:8088/wss").unwrap();
            let (mut writer, mut reader) = ws.split();
            spawn_local(async move {
                writer.send(Message::Text(String::from("WIKIT_PREVIEWER_CONNECT"))).await.unwrap();
            });
            spawn_local(async move {
                while let Some(Ok(msg)) = reader.next().await {
                    match msg {
                        Message::Text(ref cmd) if cmd.starts_with("CMD:") => {
                            if cmd.trim() == "CMD:RELOAD" {
                                if let Some(frame) = document().get_element_by_id("previewer") {
                                    if let Ok(frame) = frame.dyn_into::<HtmlIFrameElement>() {
                                        frame.set_src(&frame.src());
                                    }
                                }
                            }
                        }
                        _ => {
                        }
                    }
                }
            });
        });
        html! {
        <>
        <iframe id="previewer" onload={onload} src="http://127.0.0.1:8088" style="width: 100%; height: 100vh;" title="wikit live preview"></iframe>
        </>
        }
    }
}
