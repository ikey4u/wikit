use crate::elog;
use crate::error::{AnyResult, Context};
use crate::config;
use crate::util;

use std::path::Path;
use std::env::consts;
use std::fs::File;
use std::io::Write;
use std::collections::BTreeMap;

use dialoguer::Input;

fn ensure_mac_environment() -> AnyResult<()> {
    if consts::OS != "macos" {
        return Err(elog!("This feature is only supported on macOS"));
    }
    let confdir = config::get_config_dir()?;
    let macddk_dir = confdir.join("macddk");
    if !macddk_dir.exists() {
        let ans = loop {
            let input: String = Input::new()
                .with_prompt("[+] mac dictionary development kit is not installed, install now?")
                .with_initial_text("y")
                .default("n".into())
                .interact_text()?;
            match input.as_str() {
                "n" | "N" | "no" | "NO" => {
                    break "n";
                },
                "y" | "Y" | "yes" | "YES" => {
                    break "y";
                },
                _ => {}
            }
        };
        if ans == "n" {
            return Err(elog!("please install mac ddk manually from https://github.com/ikey4u/macddk into {}", macddk_dir.display()))
        }
        let cmd = format!("git clone https://github.com/ikey4u/macddk '{}'", macddk_dir.display());
        util::runcmd(&cmd, None).context("cannot install macddk")?;
    }

    Ok(())
}

pub fn create_mac_dictionary<I, P>(src: I, input: P, output: P, css: Option<P>) -> AnyResult<()>
    where I: Iterator<Item = (String, String)>, P: AsRef<Path>,
{
    ensure_mac_environment().context(elog!("mac environment requirements are not met"))?;

    let input = std::fs::canonicalize(input.as_ref()).context(elog!("cannot find input file"))?;
    let workdir = input.parent().context(elog!("cannot get working directory"))?.join("wikit");
    let workdir = workdir.as_path();
    std::fs::create_dir_all(workdir).context(elog!("cannot create working directory"))?;

    let input_stem = input.file_stem()
        .context(elog!("cannot get input filename"))?
        .to_str().context(elog!("cannot convert osstr to str"))?;

    let dcss = if let Some(css) = css {
        css.as_ref().to_path_buf()
    } else {
        let css_content = r#"
            @charset "UTF-8";
            @namespace d url(http://www.apple.com/DTDs/DictionaryService-1.0.rng);
        "#;
        let css = workdir.join("".to_string() + input_stem + ".css");
        let mut fcss = File::create(&css).context(elog!("cannot create css file: {}", css.display()))?;
        for line in css_content.lines() {
            let line = line.trim();
            if line.len() > 0 {
                fcss.write_all(line.as_bytes())?;
                fcss.write_all(b"\n")?;
            }
        }
        css
    };

    let dname = output.as_ref().file_stem()
        .context(elog!("cannot output filename"))?
        .to_str().context(elog!("cannot convert osstr to str"))?;

    let dsrc = {
        let xml = workdir.join("".to_string() + input_stem + ".xml");
        let mut fxml = File::create(&xml).context(elog!("cannot create xml file: {}", xml.display()))?;

        fxml.write_all(r#"<?xml version="1.0" encoding="UTF-8"?>"#.as_bytes())?;
        fxml.write_all(b"\n")?;
        fxml.write_all(r#"<d:dictionary xmlns="http://www.w3.org/1999/xhtml" xmlns:d="http://www.apple.com/DTDs/DictionaryService-1.0.rng">"#.as_bytes())?;
        fxml.write_all(b"\n")?;

        let mut bitmap = BTreeMap::new();
        let nullchar = char::from(0);
        for (word, meaning) in src {
            let (word, meaning) = (word.trim_matches(nullchar), meaning.trim_matches(nullchar));
            // Remove duplicate word
            match bitmap.get(word) {
                Some(_) => continue,
                None => bitmap.insert(word.to_string(), true),
            };
            let entry = format!(
                r#"<d:entry id="{entry_id}" d:title="{entry_title}">
                       <d:index d:value="{entry_index}"/>
                       <h1>{entry_title}</h1>
                       {entry_body}
                   </d:entry>"#,
                entry_id=word,
                entry_title=word,
                entry_index=word,
                entry_body=meaning,
            );
            for line in entry.lines() {
                let line = line.replace(r#"<?xml version="1.0" encoding="UTF-8"?>"#, "")
                    .replace("&", "&amp;");
                fxml.write_all(line.as_bytes())?;
                fxml.write_all(b"\n")?;
            }
        }

        fxml.write_all(b"</d:dictionary>")?;
        fxml.write_all(b"\n")?;

        xml
    };

    let dplist = {
        let plist = workdir.join("".to_string() + input_stem + ".plist");
        let mut fplist = File::create(&plist).context(elog!("cannot create plist file: {}", plist.display()))?;
        let content = format!(r#"
                <?xml version="1.0" encoding="UTF-8"?>
                <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
                <plist version="1.0">
                <dict>
                    <key>CFBundleDevelopmentRegion</key>
                    <string>English</string>
                    <key>CFBundleIdentifier</key>
                    <string>{identifier}</string>
                    <key>CFBundleName</key>
                    <string>{name}</string>
                    <key>CFBundleShortVersionString</key>
                    <string>{version}</string>
                    <key>DCSDictionaryCopyright</key>
                    <string>Copyright © {organization}</string>
                    <key>DCSDictionaryManufacturerName</key>
                    <string>{organization}</string>
                    <key>DCSDictionaryUseSystemAppearance</key>
                    <true/>
                </dict>
                </plist>
            "#,
            identifier = "created.by.wikit",
            name = dname,
            version = "1.0",
            organization = "wikit: https://github.com/ikey4u/wikit",
        );
        for line in content.lines() {
            let line = line.trim();
            if line.len() > 0 {
                fplist.write_all(line.as_bytes())?;
                fplist.write_all(b"\n")?;
            }
        }
        plist
    };

    let dictpath = {
        let ddkdir = config::get_config_dir()?.join("macddk");
        let builder = ddkdir.join("bin").join("build_dict.sh");
        // There must no spaces in directory DICT_DEV_KIT_OBJ_DIR
        let tmpdir = std::env::temp_dir().join("wikit");
        let tmpdir = tmpdir.as_path();
        std::fs::create_dir_all(tmpdir).context(elog!("cannot create working directory"))?;
        let tmpdir: String = format!("{}", tmpdir.display());
        let envs = vec![("DICT_DEV_KIT_OBJ_DIR".into(), tmpdir.clone())];
        let cmd = format!(
            "'{}' '{}' '{}' '{}' '{}'",
            builder.display(), dname, dsrc.display(), dcss.display(), dplist.display(),
        );
        println!("[+] Running Mac DDK ...");
        let msg = util::runcmd(&cmd, Some(envs)).context(elog!("cannot run command {}", cmd))?;
        println!("{}", msg);

        let cmds = vec![
            format!("rm -rf '{}'", output.as_ref().display()),
            format!("ditto --noextattr --norsrc '{}/{}.dictionary' '{}'", tmpdir, dname, output.as_ref().display()),
            format!("rm -rf '{}' '{}'", tmpdir, workdir.display()),
        ];
        for cmd in cmds {
            util::runcmd(&cmd, None)?;
        }

        format!("'{}'", output.as_ref().display())
    };

    println!("[+] Copy dictionary generated at {} into ~/Library/Dictionaries", dictpath);
    Ok(())
}
