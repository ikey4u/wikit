/// wikit use [toml](https://toml.io/en/) format file as its configuration, the content is simple
/// for now, as showed below
///
///     [cltcfg]
///     uris = [
///         "file://<filesystem path>",
///         "https://[user@token]<wikit api router>",
///         "http://[user@token]<wikit api router>",
///     ]
///
///     [srvcfg]
///     uris = [
///         "file://<filesystem path>",
///     ]
///     port = 8888
///     host = "0.0.0.0"
///
/// `[cltcfg]` is used for wikit desktop client, and `[srvcfg]` is used for serving dictionaries.
///
/// `uris` are a list of [URI](https://en.wikipedia.org/wiki/Uniform_Resource_Identifier) which
/// refers directory resource path, supported URIs are
///
/// - `file://`
///
///     This refers your local file on your system, such as `file:///home/user/Downloads/awesome.wikit`
///     where `/home/user/Downloads/awesome.wikit` is the full path to your dictionary
///     `awesome.wikit` on your system.
///
/// - `https://` and `http://`
///
///     Wikit allows client access remote hosted dictionary by network. However, this type URI is
///     only allowed in `[cltcfg]` section, if them does appear in `[srvcfg]`, then will be ignored.
///
///     Only IP address is allowed for now, domain will be added in future.
///
///     Assuming the base url of the remote server is `https://example.com`, then there must exist following API
///
///     - GET /wikit/list
///
///         Get dictionary list
///
///     - GET /wikit/query?word=<word>&dictname=<name>
///
///         Lookup word meaning
///
///     - GET /wikit/script?dictname=<name>
///
///         Get dictionary script
///
///     - GET /wikit/style?dictname=<name>
///
///         Get dictionary style
///
use crate::error::{AnyResult, Context};
use crate::elog;

use std::fs::{self, File};
use std::path::PathBuf;
use std::io::{Read, Write};

use dirs;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

// The max total size of MDX items (word or meaning) contained in one MDX block
pub const MAX_MDX_ITEM_SIZE: usize = (2 << 20) as usize;

pub static WIKIT_CONFIG: Lazy<crate::config::WikitConfig> = Lazy::new(|| {
    load_config().expect("Cannot load wikit config")
});

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientConfig {
    pub uris: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub uris: Vec<String>,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WikitConfig {
    pub cltcfg: ClientConfig,
    pub srvcfg: ServerConfig,
}

impl Default for WikitConfig {
    fn default() -> Self {
        WikitConfig {
            cltcfg: ClientConfig { uris: vec![] },
            srvcfg: ServerConfig { uris: vec![], host: "0.0.0.0".to_string(), port: 8888u16 },
        }
    }
}

pub fn load_config() -> AnyResult<WikitConfig> {
    let confdir = get_config_dir().context(elog!("cannot get user config directory"))?;
    let confpath = confdir.join("wikit.toml");
    if !confpath.exists() {
        File::create(&confpath)
            .context(elog!("failed to create wikit.toml"))?
            .write(toml::to_string(&WikitConfig::default())?.as_bytes())
            .context(elog!("failed to write wikit.toml"))?;
    }
    let mut fconf = File::open(&confpath).context(elog!("Failed to open config file: {:?}", confpath))?;
    let mut conf = String::new();
    fconf.read_to_string(&mut conf)?;
    let conf = toml::from_str::<WikitConfig>(&conf)?;
    Ok(conf)
}

pub fn get_config_dir() -> AnyResult<PathBuf> {
    let sysconfdir = dirs::config_dir().context(elog!("cannot get system config directory"))?;
    let confdir = sysconfdir.join("wikit");
    if !confdir.exists() {
        fs::create_dir_all(&confdir).context(elog!("failed to create {}", confdir.display()))?;
    }
    return Ok(confdir);
}

pub fn get_static_dir() -> AnyResult<PathBuf> {
    let confdir = get_config_dir()?;
    let staticdir = confdir.join("static");
    if !staticdir.exists() {
        fs::create_dir_all(&staticdir)
            .context(elog!("failed to create static directory: {}", staticdir.display()))?;
    }
    Ok(staticdir)
}
