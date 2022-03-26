use yew::prelude::*;

pub fn dynhtml(content: &str, class: Option<&str>) -> Html {
    let div = gloo::utils::document().create_element("div").expect("failed to create div tag");
    if let Some(class) = class {
        div.set_attribute("class", class).expect("failed to set class to dynhtml");
    }
    div.set_inner_html(content);
    Html::VRef(div.into())
}
