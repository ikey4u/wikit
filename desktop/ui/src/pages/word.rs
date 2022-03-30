use yew::prelude::*;

pub struct WordMsg {
}

pub struct Word;

impl Component for Word {
    type Message = WordMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <section class="section">
                <p>{ "TODO: Word Page" }</p>
            </section>
        }
    }
}
