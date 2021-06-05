mod mdict;
mod error;
mod config;
mod router;

use crate::error::{AnyResult, Context};

use std::path::Path;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::Write;

use clap::{Arg, App, SubCommand, AppSettings, value_t_or_exit};
use once_cell::sync::Lazy;
use serde::Deserialize;
use ron::de::from_reader;

static WIKIT_CONFIG: Lazy<config::WikitConfig> = Lazy::new(|| {
    config::load_config().expect("Cannot load wikit config")
});

#[derive(Debug, Deserialize)]
pub struct MDXMeta {
    title: String,
    author: String,
    description: String,
}

impl MDXMeta {
    fn default() -> Self {
        Self {
            title: "A generic MDX dictionary".to_string(),
            author: "An anonymous hero".to_string(),
            description: "Just for fun".to_string(),
        }
    }
    fn to_string(&self) -> String {
        let indent = "    ";
        return format!(
            "(\n{}\"title\": \"{}\",\n{}\"author\": \"{}\",\n{}\"description\": \"{}\",\n)",
            indent, &self.title,
            indent, &self.author,
            indent, &self.description
        );
    }
}

#[derive(Debug)]
enum ResourceFormat {
    TEXT,
    WIKIT,
    MDX,
    POSTGRES,
}

impl ResourceFormat {
    fn new(input: &str) -> Option<Self> {
        if input.starts_with("postgresql://") {
            return Some(ResourceFormat::POSTGRES);
        } else {
            match Path::new(input).extension().and_then(OsStr::to_str) {
                Some("txt") | Some("TXT") => Some(ResourceFormat::TEXT),
                Some("mdx") | Some("MDX") => Some(ResourceFormat::MDX),
                Some("wikit") | Some("WIKIT") => Some(ResourceFormat::WIKIT),
                _ => None
            }
        }
    }
}

#[rocket::main]
async fn main() -> AnyResult<()> {
    let matches = App::new("wikit")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .version("0.1.0")
        .author("ikey4u <pwnkeeper@gmail.com>")
        .about("A universal dictionary - Wikit")
        .subcommand(
            SubCommand::with_name("mdx")
            .setting(AppSettings::ArgRequiredElseHelp)
            .setting(AppSettings::ColoredHelp)
            .about("Process MDX file")
            .arg(Arg::with_name("create")
                .help("Create mdx file")
                .short("-c")
                .long("--create")
                .takes_value(false)
            )
            .arg(Arg::with_name("parse")
                .help("Parse mdx file")
                .short("-p")
                .long("--parse")
                .value_name("MDX_FILE_PATH")
                .takes_value(false)
            )
            .arg(Arg::with_name("info")
                .help("Dump basic information of mdx file")
                .long("--info")
                .takes_value(false)
            )
            .arg(Arg::with_name("metafile")
                .help(
                    format!(
                        "You could specify a meta file when create mdx file. Wikit will use default meta info if this option is not provided. The template is given below(include the parentheses):\n{}\n",
                        MDXMeta::default().to_string()
                    ).as_str()
                )
                .long("--meta")
                .takes_value(false)
            )
            .arg(Arg::with_name("output")
                .help("Same with <input>")
                .short("-o")
                .long("--output")
                .takes_value(true)
            )
            .arg(Arg::with_name("table")
                .help("The table name in the database, you must provide this parameter if input/output is a database url")
                .long("--table")
                .takes_value(true)
            )
            .arg(Arg::with_name("input")
                .help("The input file format depends on the value. If the value has a suffix .txt, then the input is a txt file; If the value has a suffix .mdx, then the input is a mdx file; If the value is a database url such as postgresql://user@localhost:5432/dictdb, then the input is a database")
                .required(true)
                .index(1)
            )
        )
        .subcommand(
            SubCommand::with_name("server")
            .setting(AppSettings::ArgRequiredElseHelp)
            .setting(AppSettings::ColoredHelp)
            .about("Run wikit as an API server")
            .arg(Arg::with_name("start")
                .help("Start server")
                .short("-s")
                .long("--start")
                .takes_value(false)
            )
        )
        .get_matches();

    if let Some(mdx) = matches.subcommand_matches("mdx") {
        let input = value_t_or_exit!(mdx.value_of("input"), String);

        if mdx.is_present("info") {
            mdict::parse_mdx(&input, Some(mdict::ParseOption::OnlyHeader))?;
        } else {

            let output = value_t_or_exit!(mdx.value_of("output"), String);
            let itype  = ResourceFormat::new(&input).ok_or(elog!("Failed to get input resource format"))?;
            let otype = ResourceFormat::new(&output).ok_or(elog!("Failed to get output resource format"))?;

            if mdx.is_present("create") {
                let meta = match mdx.value_of("metafile") {
                    None => MDXMeta::default(),
                    Some(path) => {
                        let path = Path::new(path);
                        let metafile = File::open(path)
                            .context(elog!("Failed to open meta file: {}", path.display()))?;
                        from_reader(metafile).context("Failed to deserialize meat file")?
                    }
                };
                match (itype, otype) {
                    (ResourceFormat::TEXT, ResourceFormat::MDX) => {
                        mdict::create_mdx(&meta.title, &meta.author, &meta.description, &input, &output)?;
                    },
                    (i, o) => {
                        return Err(elog!("Does not support creating {:?} from {:?} for now", o, i));
                    },
                }
            } else if mdx.is_present("parse") {
                match (itype, otype) {
                    (ResourceFormat::MDX, ResourceFormat::TEXT) => {
                        let dict = mdict::parse_mdx(input.as_str(), None)?;
                        let mut dstmdx = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(true)
                            .open(&output)
                            .context(elog!("Cannot open {:?}", &output))?;
                        for (word, meaning) in dict {
                            let item = format!("{}\n{}\n</>\n", word, meaning);
                            dstmdx.write(item.as_bytes())?;
                        }
                    },
                    (i, o) => {
                        return Err(elog!("Does not support parse {:?} into {:?} for now", i, o));
                    }
                }
            } else {
                println!("No valid flags are provided, usage: {}", matches.usage());
            }

            if mdx.is_present("table") {
                let table = mdx.value_of("table").expect("Please specify database table name");
                let dict = mdict::parse_mdx(input.as_str(), None)?;
                mdict::save_into_db(dict, &WIKIT_CONFIG.dburl, table).await?;
            }
        }

    }

    if let Some(server) = matches.subcommand_matches("server") {
        if server.is_present("start") {
            // The database config is read from $HOME/.config/wikit/wikit.ron
            router::rocket().launch().await?;
        }
    }

    Ok(())
}
