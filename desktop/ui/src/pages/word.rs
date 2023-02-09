use crate::{dom, ffi::{self, get_dict_list}};
use crate::util;

use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use wikit_proto::{DictMeta, LookupResponse};
use web_sys::HtmlSelectElement;

const DEBOUNCE_DELTA: i64 = 50;

pub enum WordMsg {
    OnSearchTextChange(String),
    OnClickFuzzyItem(String),
    StartLookup,
    OnLookupResult(LookupResponse),
    OnDictMetaList(Vec<DictMeta>),
    OnDictChange(usize),
}

pub struct Word {
    input: String,
    fuzzy_list: Vec<String>,
    current_dictionary_index: Option<usize>,
    dict_meta_list: Vec<DictMeta>,
    previous_change_epoch: i64,
    cache: Option<LookupResponse>,
    show_meaning: bool,
    word_meaning: String,
    style: String,
    script: String,
}

enum Field {
    SearchInput,
}

impl Field {
    pub fn get(&self) -> String {
        match self {
            Field::SearchInput => {
                dom::get_input_value("search")
            }
        }
    }
    pub fn set<S: AsRef<str>>(&self, val: S) {
        match self {
            Field::SearchInput => {
                dom::set_input_value("search", val.as_ref());
            }
        }
    }
}

impl Component for Word {
    type Message = WordMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link().clone();
        spawn_local(async move {
            let metas: Vec<DictMeta> = get_dict_list().await.expect_throw("failed to get dictionary list")
                .into_serde().expect_throw("dictionary list corrupted");
            link.send_message(WordMsg::OnDictMetaList(metas));
        });
        Self {
            input: String::new(),
            fuzzy_list: vec![],
            current_dictionary_index: None,
            dict_meta_list: vec![],
            previous_change_epoch: util::get_epoch_millis(),
            cache: None::<LookupResponse>,
            show_meaning: false,
            word_meaning: "".to_owned(),
            style: "".to_owned(),
            script: "".to_owned(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let link = ctx.link().clone();
        match msg {
            WordMsg::OnSearchTextChange(_) => {
                self.input = Field::SearchInput.get().to_lowercase();
                let epoch = util::get_epoch_millis();
                if epoch - self.previous_change_epoch >= DEBOUNCE_DELTA && self.input.trim().len() > 0 {
                    link.send_message(WordMsg::StartLookup);
                }
            }
            WordMsg::StartLookup => {
                self.previous_change_epoch = util::get_epoch_millis();
                self.show_meaning = false;
                if let Some(idx) = self.current_dictionary_index {
                    let dictid = self.dict_meta_list[idx].id.clone();
                    let input = self.input.clone();
                    spawn_local(async move {
                        let cache: LookupResponse = ffi::lookup(dictid, input).await.expect_throw("failed to lookup")
                            .into_serde().expect_throw("lookup up response corrupted");
                        link.send_message(WordMsg::OnLookupResult(cache));
                    });
                }
            }
            WordMsg::OnClickFuzzyItem(word) => {
                if let Some(r) = self.cache.as_ref() {
                    if let Some(meaning) = r.words.get(word.as_str()) {
                        Field::SearchInput.set(word);
                        self.word_meaning = meaning.to_owned();
                        self.show_meaning = true;
                    }
                }
            }
            WordMsg::OnLookupResult(r) => {
                let mut fuzzy_list = r.words.keys().map(|v| v.clone()).collect::<Vec<String>>();
                fuzzy_list.sort();
                if fuzzy_list.contains(&self.input) {
                    if let Some(meaning) = r.words.get(&self.input) {
                        self.word_meaning = meaning.to_owned();
                        self.show_meaning = true;
                        self.style = r.style.clone();
                        self.script = r.script.clone();
                    }
                } else {
                    self.fuzzy_list = fuzzy_list;
                }
                self.cache = Some(r);
            }
            WordMsg::OnDictMetaList(metas) => {
                if metas.len() > 0 {
                    self.current_dictionary_index = Some(0);
                }
                self.dict_meta_list = metas;
            }
            WordMsg::OnDictChange(dictidx) => {
                self.current_dictionary_index = Some(dictidx);
                link.send_message(WordMsg::StartLookup);
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let on_dictionary_change = link.callback(|e: Event| {
            let select = e.target_dyn_into::<HtmlSelectElement>().unwrap();
            WordMsg::OnDictChange(select.value().parse::<usize>().unwrap())
        });
        let ui_search_bar = {
            html! {
                <div class="field has-addons">
                    <p class="control">
                      <span class="select is-rounded is-small">
                        <select onchange={on_dictionary_change}> {
                            self.dict_meta_list.iter().enumerate().map(|(i, meta)| {
                                html! {
                                    <option value={i.to_string()}>{ meta.name.clone() }</option>
                                }
                            }).collect::<Html>()
                        } </select>
                      </span>
                    </p>
                    <p class="control has-icons-left is-expanded">
                        <input
                            class="input is-rounded is-small"
                            autocomplete="none" autocorrect="off" autocapitalize="none"
                            type="text"
                            id="search"
                            onkeyup={
                                ctx.link().callback(|_| {
                                    WordMsg::OnSearchTextChange(Field::SearchInput.get())
                                })
                            }
                        />
                        <span class="icon is-small is-left">
                            <i class="bi bi-search"></i>
                        </span>
                    </p>
                </div>
            }
        };
        let ui_no_dictionary_found = {
            html! {
                <div>
                    <p>{ "No dictionary is found" }</p>
                </div>
            }
        };
        let ui_meaning = {
            let content = format!(r#"
                <!DOCTYPE html>
                <html>
                  <head>
                    <meta charset="UTF-8" />
                    {script}
                    {style}
                  </head>
                  <body>
                    {body}
                  </body>
                </html>
            "#,
                script = self.script,
                style = self.style,
                body = self.word_meaning,
            );
            html! {
                <iframe id="scrollbar-arrow" class="fill-xy" style="overflow-x: hidden; overflow-y: auto;" srcdoc={content}></iframe>
            }
        };
        let ui_word_candidates = {
            html! {
                <div class="column wikit-list"> {
                    self.fuzzy_list.clone().into_iter().map(|item| {
                        html!{
                            <div>
                                <button class="button is-fullwidth" onclick={
                                    let item = item.clone();
                                    ctx.link().callback(move |_| {
                                        WordMsg::OnClickFuzzyItem(item.clone())
                                    })
                                }>
                                { item }
                                </button>
                            </div>
                        }
                    }).collect::<Html>()
                } </div>
            }
        };
        let ui_no_word_candidate = {
            html! {
                <div class="fill-xy has-text-centered pt-6">{"Nothing is there ..."}</div>
            }
        };
        let ui_meaning_placeholder = {
            html! {
                <div class="fill-xy has-text-centered pt-6">{"Type a word to look up ..."}</div>
            }
        };
        html! {
            <div class="fill-xy">
                if self.dict_meta_list.len() > 0 {
                    {ui_search_bar}
                    if self.input.trim().len() > 0 {
                        if self.show_meaning {
                            {ui_meaning}
                        } else {
                            if self.fuzzy_list.len() > 0 {
                                {ui_word_candidates}
                            } else {
                                {ui_no_word_candidate}
                            }
                        }
                    } else {
                        {ui_meaning_placeholder}
                    }
                } else {
                    {ui_no_dictionary_found}
                }
            </div>
        }
    }
}
