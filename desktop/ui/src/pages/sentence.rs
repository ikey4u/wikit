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
                <div class="field is-grouped">
                    <p class="control is-expanded">
                        <input class="input" type="text" placeholder="Search sentence ..." />
                    </p>
                    <p class="control">
                        <a class="button is-light">
                            { "Search" }
                        </a>
                    </p>
                </div>
            </section>
        }
    }
}
