use yew::prelude::*;

pub struct SettingMsg {
}

pub struct Setting;

impl Component for Setting {
    type Message = SettingMsg;
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
                    <p>{ "TODO: Setting Page" }</p>
                </div>
            </section>
        }
    }
}
