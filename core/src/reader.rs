use crate::config::MAX_MDX_ITEM_SIZE;
use crate::util;
use crate::error::Result;

use std::io::{BufReader, BufRead, Lines};
use std::fs::File;

use serde::{Deserialize, Serialize};

pub struct MDXSource {
    pub iter: Lines<BufReader<File>>,
}

impl MDXSource {
    pub fn new(f: File) -> Self {
        let reader = BufReader::new(f);
        MDXSource { iter: reader.lines() }
    }
}

impl Iterator for MDXSource {
    type Item = (String, String);
    fn next(&mut self) -> Option<Self::Item> {
        let (mut word, mut meaning) = (String::new(), String::new());
        loop {
            match self.iter.next() {
                Some(line) => match line {
                    Ok(line) => {
                        if line.trim() == "</>" {
                            break;
                        }
                        if word.len() == 0 {
                            word = line.trim().to_string();
                        } else {
                            meaning.push_str(line.as_str().trim());
                        }
                    },
                    Err(_) => return None
                },
                None => return None
            }
        }
        let mut word = if word.len() > MAX_MDX_ITEM_SIZE {
            println!("[!] Lenght of word exceeds {}, truncated!", MAX_MDX_ITEM_SIZE);
            word[..MAX_MDX_ITEM_SIZE].to_string()
        } else {
            word
        };
        word.push(0 as char);
        let mut meaning = if meaning.len() > MAX_MDX_ITEM_SIZE {
            println!("[!] Lenght of meaning exceeds {}, truncated!", MAX_MDX_ITEM_SIZE);
            meaning[..MAX_MDX_ITEM_SIZE].to_string()
        } else {
            meaning
        };
        meaning.push(0 as char);
        return Some((util::normalize_word(word), meaning));
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WikitSourceItemHeader {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: String,
    pub mime: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WikitSourceItem {
    pub header: WikitSourceItemHeader,
    pub body: String,
}

pub struct WikitSource {
    lineno: usize,
    pub iter: Lines<BufReader<File>>,
}

impl WikitSource {
    pub fn new(f: File) -> Self {
        let reader = BufReader::new(f);
        WikitSource {
            iter: reader.lines(),
            lineno: 0,
        }
    }
}

impl Iterator for WikitSource {
    type Item = WikitSourceItem;
    fn next(&mut self) -> Option<Self::Item> {
        #[derive(PartialEq)]
        enum Status {
            Idle,
            ReadingHeader,
            ReadingBody,
        }
        let mut status = Status::Idle;
        let (mut hdrstr, mut bodystr) = (String::new(), String::new());
        let item_start_lineno = self.lineno + 1;
        loop {
            let line = self.iter.next();
            if let Some(Ok(line)) = line {
                self.lineno += 1;
                let indent = line.len() - line.trim_start().len();
                let mut line = line.trim_end();
                // status transition
                match status {
                    Status::Idle => {
                        // header start flag
                        if line == "(" {
                            status = Status::ReadingHeader;
                            continue;
                        }
                    }
                    Status::ReadingHeader => {
                        // body start flag: start with `)`, following with zero or more space, end with `{`
                        if line.starts_with(")") && line.ends_with("{") && line.replace(" ", "").trim() == "){" {
                            status = Status::ReadingBody;
                            continue;
                        }
                    }
                    Status::ReadingBody => {
                        // body end flag
                        if line == "}" {
                            break;
                        }
                    }
                }
                // execute status
                match status {
                    Status::ReadingHeader | Status::ReadingBody => {
                        let line = if indent == 0 || indent >= 4 {
                            if line.len() >= 4 && indent >= 4 {
                                &line[4..]
                            } else {
                                &line[..]
                            }
                        } else {
                            println!("line {}: 0 or more than 4 spaces indent are expected", self.lineno);
                            return None;
                        };
                        if status == Status::ReadingHeader {
                            hdrstr += line;
                            hdrstr += "\n";
                        } else {
                            bodystr += line;
                            bodystr += "\n";
                        }
                    }
                    _ => {}
                }
            } else {
                return None;
            }
        }

        if hdrstr.len() > 0 {
            let hdrstr = format!("{{ {hdrstr} }}");
            match json5::from_str::<WikitSourceItemHeader>(hdrstr.as_str()) {
                Ok(header) => {
                    return Some(WikitSourceItem { header, body: bodystr });
                }
                Err(e) => {
                    println!("failed to parse header from line {} to {}, content:\n{}\n, with error:\n{}", item_start_lineno, self.lineno, hdrstr, e);
                }
            }
        }

        return None;
    }
}
