mod mdict;
mod error;
mod config;
mod router;

use crate::error::{AnyResult};

use clap::{Arg, App, SubCommand, AppSettings};
use once_cell::sync::Lazy;

static WIKIT_CONFIG: Lazy<config::WikitConfig> = Lazy::new(|| {
    config::load_config().expect("Cannot load wikit config")
});

#[rocket::main]
async fn main() -> AnyResult<()> {
    let matches = App::new("wikit")
        .setting(AppSettings::ArgRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .version("0.0.1")
        .author("ikey4u <pwnkeeper@gmail.com>")
        .about("A universal dictionary - Wikit")
        .subcommand(SubCommand::with_name("mdx")
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .setting(AppSettings::ColoredHelp)
                    .about("Process MDX file")
                    .arg(Arg::with_name("input")
                        .short("i")
                        .long("--input")
                        .value_name("MDX_FILE_PATH")
                        .takes_value(true))
                    .arg(Arg::with_name("table")
                         .help("Parse <MDX_FILE_PATH> and save to <TABLE_NAME> of database")
                         .short("t")
                         .long("--table")
                         .value_name("TABLE_NAME")
                         .takes_value(true))
                    .arg(Arg::with_name("info")
                         .help("Dump basic information of <MDX_FILE_PATH>")
                         .long("--info")
                         .value_name("INFO")
                         .takes_value(false))
                    )
        .subcommand(SubCommand::with_name("server")
                    .setting(AppSettings::ArgRequiredElseHelp)
                    .setting(AppSettings::ColoredHelp)
                    .about("Control backend API server")
                    .arg(Arg::with_name("start")
                         .help("start server")
                         .short("s")
                         .long("--start"))
                    )
        .get_matches();

    if let Some(mdx) = matches.subcommand_matches("mdx") {
        let input = mdx.value_of("input").expect("Please specify the MDX file path");
        if mdx.is_present("table") {
            let table = mdx.value_of("table").expect("Please specify database table name");
            let dict = mdict::parse_mdx(input, None)?;
            mdict::save_into_db(dict, &WIKIT_CONFIG.dburl, table).await?;
        }
        if mdx.is_present("info") {
            mdict::parse_mdx(input, Some(mdict::ParseOption::OnlyHeader)).expect("Unable to parse MDX");
        }
    }

    if let Some(server) = matches.subcommand_matches("server") {
        if server.is_present("start") {
            router::rocket().launch().await?;
        }
    }

    Ok(())
}
