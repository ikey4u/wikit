use crate::wikit;

use std::{sync::Mutex, collections::HashMap};
use std::path::Path;
use std::sync::Arc;

use once_cell::sync::Lazy;

#[derive(Debug)]
pub enum CacheValue {
    StringList(Vec<String>),
    WikitDictionary(wikit::WikitDictionary),
}

pub static CACHE: Lazy<Arc<Mutex<HashMap<String, CacheValue>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

fn query(word: String, dictname: String) -> String {
    let dictkey = format!("server.dict://{}", dictname);

    if let Ok(c) = CACHE.lock() {
        if let Some(CacheValue::WikitDictionary(dict)) = c.get(dictkey.as_str()) {
            match dict {
                wikit::WikitDictionary::Local(d) => {
                },
                wikit::WikitDictionary::Remote(d) => {
                },
                _ => {}
            }
        } else {
        }
    }

    "".to_string()
}
