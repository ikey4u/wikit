use yew::prelude::*;

pub struct FavoriteMsg {
}

pub struct Favorite;

impl Component for Favorite {
    type Message = FavoriteMsg;
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
                <p>{ "TODO: Favorite Page" }</p>
            </section>
        }
    }
}
