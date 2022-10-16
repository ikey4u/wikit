use crate::{dom, ffi::{self, get_dict_list}};
use crate::util;

use wasm_bindgen::{JsCast, UnwrapThrowExt};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use wikit_proto::{DictMeta, LookupResponse};
use gloo_utils::format::JsValueSerdeExt;

const DEBOUNCE_DELTA: i64 = 500;

pub enum WordMsg {
    OnSearchTextChange(String),
    OnLookup(String),
    OnLookupResult(LookupResponse),
    OnDictMetaList(Vec<DictMeta>),
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
}

enum Fileds {
    SearchInput,
}

impl Fileds {
    pub fn value(&self) -> String {
        match self {
            Fileds::SearchInput => {
                dom::get_input_value("search")
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
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let link = ctx.link().clone();
        match msg {
            WordMsg::OnSearchTextChange(_) => {
                let input = Fileds::SearchInput.value();
                if util::get_epoch_millis() - self.previous_change_epoch >= DEBOUNCE_DELTA && input.trim().len() > 0 {
                    self.input = input.clone();
                    self.previous_change_epoch = util::get_epoch_millis();
                    self.show_meaning = false;
                    if let Some(idx) = self.current_dictionary_index {
                        let dictid = self.dict_meta_list[idx].id.clone();
                        spawn_local(async move {
                            let cache: LookupResponse = ffi::lookup(dictid, input).await.expect_throw("failed to lookup")
                                .into_serde().expect_throw("lookup up response corrupted");
                            link.send_message(WordMsg::OnLookupResult(cache));
                        });
                    }
                }
            }
            WordMsg::OnLookup(word) => {
                if let Some(r) = self.cache.as_ref() {
                    if let Some(meaning) = r.words.get(word.as_str()) {
                        self.word_meaning = meaning.to_owned();
                        self.show_meaning = true;
                    }
                }
            }
            WordMsg::OnLookupResult(r) => {
                self.fuzzy_list = r.words.keys().map(|v| v.clone()).collect();
                self.cache = Some(r);
            }
            WordMsg::OnDictMetaList(metas) => {
                if metas.len() > 0 {
                    self.current_dictionary_index = Some(0);
                }
                self.dict_meta_list = metas;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
         let meaning_page = if let Some(r) = self.cache.as_ref() {
            let page = format!(r#"
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
                script = r.script,
                style = r.style,
                body = self.word_meaning,
            );
            html! {
                <iframe title="dictview" srcdoc={page} style="width: 100%; height: 548px;"></iframe>
            }
        } else {
            html! {
                <p>{"bug found!panic"}{self.input.clone()}</p>
            }
        };

        html! {
            <div>
                if self.dict_meta_list.len() > 0 {
                    <div class="field has-addons">
                        <p class="control">
                          <span class="select is-rounded is-small">
                            <select> {
                                self.dict_meta_list.iter().map(|meta| {
                                    html! {
                                        <option>{ meta.name.clone() }</option>
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
                                        WordMsg::OnSearchTextChange(Fileds::SearchInput.value())
                                    })
                                }
                            />
                            <span class="icon is-small is-left">
                                <i class="bi bi-search"></i>
                            </span>
                        </p>
                    </div>
                } else {
                    <div>
                        <p>{ "No dictionary is found" }</p>
                    </div>
                }
                if self.show_meaning {
                    {meaning_page}
                } else {
                    if self.fuzzy_list.len() > 0 {
                        <div class="column wikit-list"> {
                            self.fuzzy_list.clone().into_iter().map(|item| {
                                html!{
                                    <div>
                                        <button class="button is-fullwidth" onclick={
                                            let item = item.clone();
                                            ctx.link().callback(move |_| {
                                                WordMsg::OnLookup(item.clone())
                                            })
                                        }>
                                        { item }
                                        </button>
                                    </div>
                                }
                            }).collect::<Html>()
                        } </div>
                    }
                }
            </div>
        }
    }
}
