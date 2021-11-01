use crate::error::{AnyResult, Context};
use crate::elog;

use std::fs::{self, File};
use std::collections::HashMap;
use std::path::PathBuf;
use std::io::{Write};

use dirs;
use ron::de::from_reader;
use serde::Deserialize;

// The max total size of MDX items (word or meaning) contained in one MDX block
pub const MAX_MDX_ITEM_SIZE: usize = (2 << 20) as usize;

pub const WIKIT_DEFAULT_CONFIG: &str = r#"
(
    dburl: "postgresql://user@localhost:5432/dictdb",
    dict: {
        "dictionary_oxford": "dictionary_oxford_table",
        "dictionary_collins": "dictionary_collins_table",
    },
    // local contains a list of absolute path (directory or wikit dictionary path)
    local: [
        "",
    ],
    // server contains a list of dictionary server API
    server: [
        (
            name: "",
            api: "",
        )
    ],
)
"#;

#[derive(Debug, Deserialize)]
pub struct WikitServer {
    name: String,
    api: String,
}

#[derive(Debug, Deserialize)]
pub struct WikitConfig {
    pub dburl: String,
    pub dict: HashMap<String, String>,
    pub server: Vec<WikitServer>,
    pub local: Vec<String>,
}

pub fn load_config() -> AnyResult<WikitConfig> {
    let confdir = get_config_dir().context(elog!("cannot get user config directory"))?;
    if !confdir.exists() {
        fs::create_dir_all(&confdir).context(elog!("failed to create {}", confdir.display()))?;
    }
    let confpath = confdir.join("wikit.ron");
    if !confpath.exists() {
        File::create(&confpath)
            .context(elog!("failed to create wikit.ron"))?
            .write(WIKIT_DEFAULT_CONFIG.as_bytes())
            .context(elog!("failed to write wikit.ron"))?;
    }
    let f = File::open(&confpath)
        .context(
            elog!(
                "Failed to open config file: {:?}, please create it with following content: {}",
                confpath, WIKIT_DEFAULT_CONFIG
            )
        )?;
    let wikit_config: WikitConfig = from_reader(f)
        .context(elog!("Cannot load config file: {:?}", confpath))?;
    Ok(wikit_config)
}

// TODO(2021-11-01): depracated, to be refactored out
pub fn init_config_dir() -> AnyResult<()> {
    let confdir = get_config_dir().context(elog!("cannot get user config directory"))?;
    let confdir = confdir.as_path();
    if !confdir.exists() {
        fs::create_dir_all(confdir).context(elog!("failed to create {}", confdir.display()))?;
    }
    Ok(())
}

pub fn get_config_dir() -> AnyResult<PathBuf> {
    let sysconfdir = dirs::config_dir().context(elog!("cannot get system config directory"))?;
    let confdir = sysconfdir.join("wikit");
    return Ok(confdir);
}
