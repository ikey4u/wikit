use yew::prelude::*;

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
        html! {
        <>
            <iframe src="http://127.0.0.1:8088" style="width: 100%; height: 100vh;" title="wikit live preview"></iframe>
        </>
        }
    }
}
