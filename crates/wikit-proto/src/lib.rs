use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DictMeta {
    pub name: String,
    pub id: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LookupResponse {
    // possible of (word, meaning) pair list
    pub words: HashMap<String, String>,
    // html js tag
    pub script: String,
    // html css tag
    pub style: String,
}
