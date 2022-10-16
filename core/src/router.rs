use crate::config;
use crate::wikit;
use crate::crypto;

use std::net::{IpAddr, Ipv4Addr};
use std::{sync::Mutex, collections::HashMap};
use std::sync::Arc;

use rocket::{Build, Request, catch, get, catchers, routes};
use rocket::serde::json::Json;
use once_cell::sync::Lazy;
use wikit_proto::DictMeta;

pub static DICTMP: Lazy<Arc<Mutex<HashMap<String, String>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});


#[catch(500)]
fn internal_error() -> &'static str {
    "It seems that you are not on earth"
}

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("I couldn't find '{}'. Try something else?", req.uri())
}

#[get("/list")]
async fn list() -> Json<Vec<DictMeta>> {
    let mut dictlist = vec![];
    if let Ok(config) = config::load_config() {
        for uri in config.srvcfg.uris.iter() {
            let dictid = crypto::md5(uri.as_bytes());
            if let Some(dict) = wikit::load_dictionary_from_uri(uri) {
                let style_key = format!("style[{}]", dictid);
                let script_key = format!("script[{}]", dictid);
                if let wikit::WikitDictionary::Local(d) = dict {
                    if let Ok(mut dictmp) = DICTMP.lock() {
                        if dictmp.get(&style_key).is_none() {
                            dictmp.insert(style_key, d.head.style.clone());
                        }
                        if dictmp.get(&script_key).is_none() {
                            dictmp.insert(script_key, d.head.script.clone());
                        }
                    }
                    if let Ok(mut dictmp) = DICTMP.lock() {
                        dictmp.insert(dictid.clone(), uri.to_string());
                    }

                    dictlist.push(DictMeta {
                        name: d.head.name,
                        id: dictid.clone(),
                    });
                }
            }
        }
    }
    Json(dictlist)
}

#[get("/style?<dictname>")]
async fn style(dictname: String) -> String {
    if let Ok(dictmp) = DICTMP.lock() {
        let style_key = format!("style[{}]", dictname);
        if let Some(style) = dictmp.get(style_key.as_str()) {
            return style.to_string();
        }
    }
    "".to_string()
}

#[get("/script?<dictname>")]
async fn script(dictname: String) -> String {
    let script_key = format!("script[{}]", dictname);
    if let Ok(dictmp) = DICTMP.lock() {
        if let Some(script) = dictmp.get(script_key.as_str()) {
            return script.to_string();
        }
    }
    "".to_string()
}

#[get("/query?<word>&<dictname>")]
async fn query(word: String, dictname: String) -> Json<Vec<(String, String)>> {
    let r = vec![];
    if let Ok(dictmp) = DICTMP.lock() {
        if let Some(uri) = dictmp.get(dictname.as_str()) {
            if let Some(dict) = wikit::load_dictionary_from_uri(uri) {
                match dict {
                    wikit::WikitDictionary::Local(d) => {
                        if let Ok(r) = d.lookup(word) {
                            return Json(r);
                        }
                    },
                    wikit::WikitDictionary::Remote(d) => {
                        if let Ok(r) = d.lookup(&word, &dictname) {
                            return Json(r);
                        }
                    },
                }
            }
        }
    }
    return Json(r);
}

pub fn rocket() -> rocket::Rocket<Build> {
    let cfg = match config::load_config() {
        Ok(cfg) => {
            rocket::Config {
                port: cfg.srvcfg.port,
                address: cfg.srvcfg.host.parse().unwrap_or(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))),
                ..rocket::Config::debug_default()
            }
        },
        Err(e) => {
            println!("failed to load config with error: {:?}", e);
            rocket::Config::default()
        }
    };
    rocket::custom(&cfg)
        .mount("/wikit/", routes![query, list, style, script])
        .register("/", catchers![internal_error, not_found])
}
