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
        let page = format!(r#"
            <!DOCTYPE html>
            <html>
              <head>
                <meta charset="UTF-8" />
                <script type="text/javascript"> {script} </script>
                <style type="text/css" media="screen"> {style} </style>
              </head>
              <body>
                {body}
              </body>
            </html>
        "#,
            script = "",
            style = "",
            body = "",
        );
        html! {
            <iframe title="dictview" srcdoc={page} style="width: 100%; height: 100vh;"></iframe>
        }
    }
}
