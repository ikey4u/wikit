use crate::config::MAX_MDX_ITEM_SIZE;
use crate::util;

use std::io::{BufReader, BufRead, Lines};
use std::fs::{File};

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
