use yew::prelude::*;

pub struct SentenceMsg {
}

pub struct Sentence;

impl Component for Sentence {
    type Message = SentenceMsg;
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
                <p>{ "TODO: Sentence Page" }</p>
            </section>
        }
    }
}
