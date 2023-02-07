use yew::prelude::*;
use web_sys::HtmlInputElement;
use gloo::utils::document;
use wasm_bindgen::JsCast;

pub fn dynhtml(content: &str, class: Option<&str>) -> Html {
    let div = gloo::utils::document().create_element("div").expect("failed to create div tag");
    if let Some(class) = class {
        div.set_attribute("class", class).expect("failed to set class to dynhtml");
    }
    div.set_inner_html(content);
    Html::VRef(div.into())
}

pub fn get_input_value(id: &str) -> String {
    if let Some(element) = document().get_element_by_id(id) {
        if let Some(element) = element.dyn_ref::<HtmlInputElement>() {
            return element.value();
        }
    }
    "".into()
}

pub fn set_input_value(id: &str, val: &str) {
    if let Some(element) = document().get_element_by_id(id) {
        if let Some(element) = element.dyn_ref::<HtmlInputElement>() {
            element.set_value(val);
        }
    }
}
