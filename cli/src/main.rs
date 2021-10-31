use wikit_core::mdict;
use wikit_core::config;
use wikit_core::router;
use wikit_core::mac;
use wikit_core::reader;
use wikit_core::util;
use wikit_core::elog;
use wikit_core::error::{AnyResult, Context};

use std::path::Path;
use std::ffi::OsStr;
use std::fs::File;

use clap::{Arg, App, SubCommand, AppSettings, value_t_or_exit};
use serde::Deserialize;
use ron::de::from_reader;

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
            "(\n{}title: \"{}\",\n{}author: \"{}\",\n{}description: \"{}\",\n)",
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
    MACDICT,
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
                Some("dictionary") => Some(ResourceFormat::MACDICT),
                _ => None
            }
        }
    }
}

#[rocket::main]
async fn main() -> AnyResult<()> {
    config::init_config_dir().context(elog!("cannot init wikit config directory"))?;

    let matches = App::new("wikit")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .version("0.2.2")
        .author("ikey4u <pwnkeeper@gmail.com>")
        .about("A universal dictionary - Wikit")
        .subcommand(
            SubCommand::with_name("dict")
            .setting(AppSettings::ArgRequiredElseHelp)
            .setting(AppSettings::ColoredHelp)
            .about("Process dictionary file")
            .arg(Arg::with_name("create")
                .help("Create dictionary file")
                .short("-c")
                .long("--create")
                .takes_value(false)
            )
            .arg(Arg::with_name("info")
                .help("Dump basic information of dictionary file")
                .long("--info")
                .takes_value(false)
            )
            .arg(Arg::with_name("metafile")
                .help(
                    format!(
                        "You could specify a meta file when create dictionary file. Wikit will use default meta info if this option is not provided. The template is given below(include the parentheses):\n{}\n",
                        MDXMeta::default().to_string()
                    ).as_str()
                )
                .long("--meta")
                .takes_value(true)
            )
            .arg(Arg::with_name("output")
                .help("Same with <input>")
                .short("-o")
                .long("--output")
                .takes_value(true)
            )
            .arg(Arg::with_name("css")
                .help("Path of the CSS file")
                .long("--css")
                .takes_value(true)
            )
            .arg(Arg::with_name("table")
                .help("The table name in the database, you must provide this parameter if input/output is a database url")
                .long("--table")
                .takes_value(true)
            )
            .arg(Arg::with_name("input")
                .help("The input file format depends on the value. File suffix reflects the format: .txt => text, .mdx => mdx. If the value is a database url such as postgresql://user@localhost:5432/dictdb, then the input is a database")
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

    if let Some(dict) = matches.subcommand_matches("dict") {
        let input = value_t_or_exit!(dict.value_of("input"), String);
        let itype  = ResourceFormat::new(&input).ok_or(elog!("Failed to get input resource format"))?;
        if dict.is_present("info") {
            match itype {
                ResourceFormat::MDX => {
                    mdict::parse_mdx(&input, Some(mdict::ParseOption::OnlyHeader))?;
                },
                _ => {
                    println!("Dump information for this dictionary type is not supported now")
                }
            }
        } else {
            let output = value_t_or_exit!(dict.value_of("output"), String);
            let otype = ResourceFormat::new(&output).ok_or(elog!("Failed to get output resource format"))?;
            let css = if let Some(css) = dict.value_of("css") {
                Some(css.to_string())
            } else {
                None
            };

            if dict.is_present("create") {
                let meta = match dict.value_of("metafile") {
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
                    (ResourceFormat::MDX, ResourceFormat::TEXT) => {
                        let dict = mdict::parse_mdx(input.as_str(), None)?;
                        mdict::write_into_text(dict, &output)?;
                    },
                    (ResourceFormat::TEXT, ResourceFormat::MACDICT) => {
                        let file = File::open(&input).context(elog!("Cannot open {:?}", &input))?;
                        let mdxsrc = reader::MDXSource::new(file);
                        mac::create_mac_dictionary(mdxsrc, input, output, css)
                            .context(elog!("Failed to create mac dictionary"))?;
                    },
                    (ResourceFormat::MDX, ResourceFormat::MACDICT) => {
                        let (pdir, stem, _suffix) = util::parse_path(input.as_str())
                            .context(elog!("failed to get path of input file: {}", input))?;
                        let textpath = pdir.join(stem + "_wikit.txt");
                        if !textpath.exists() {
                            let dict = mdict::parse_mdx(input.as_str(), None)?;
                            if mdict::write_into_text(dict, textpath.as_path()).is_err() {
                                std::fs::remove_file(textpath.as_path())
                                    .context(elog!("cannot remove file {}", textpath.display()))?;
                            };
                        }
                        println!("[+] Open mdx soure file ...");
                        let file = File::open(&textpath)
                            .context(elog!("Cannot open {:?}", textpath.display()))?;
                        let mdxsrc = reader::MDXSource::new(file);
                        println!("[+] Create mac dictionary ...");
                        mac::create_mac_dictionary(mdxsrc, input, output, css)
                            .context(elog!("Failed to create mac dictionary"))?;
                        println!("[+] Create mac dictionary is done");
                        if textpath.exists() {
                            std::fs::remove_file(textpath.as_path()).context(
                                elog!("cannot remove file {}, you may remove it manually", textpath.display())
                            )?;
                        }
                    },
                    (ResourceFormat::MDX, ResourceFormat::POSTGRES) => {
                        let table = dict.value_of("table").expect("Please specify database table name");
                        let pairs = mdict::parse_mdx(input.as_str(), None)?;
                        mdict::save_into_db(pairs, &output, table).await?;
                    }
                    (i, o) => {
                        return Err(elog!("Does not support creating {:?} from {:?} for now", o, i));
                    },
                }
            } else {
                println!("No valid flags are provided, usage: {}", matches.usage());
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