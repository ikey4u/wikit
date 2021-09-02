use crate::elog;
use crate::error::{AnyResult, Context};

use std::process::Command;

struct ArgParser<'a> {
    buf: &'a str,
    consumed: usize,
    total: usize,
}

impl<'a> ArgParser<'a> {
    fn new(buf: &'a str) -> Self {
        ArgParser {
            buf: buf,
            consumed: 0,
            total: buf.chars().count(),
        }
    }
}

impl<'a> Iterator for ArgParser<'a> {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        if self.consumed >= self.total {
            return None;
        }

        let mut arg = String::new();
        let (mut quote, mut quotech) = (false, ' ');
        for c in self.buf.chars().skip(self.consumed) {
            self.consumed += 1;
            // take string until the next same `c` or end
            if quote || c == '\'' || c == '\"' {
                if quote {
                    if c == quotech {
                        quote = false;
                    } else {
                        arg.push(c);
                    }
                } else {
                    quote = true;
                    quotech = c;
                }
            } else if c == ' ' {
                if !quote {
                    break;
                }
            } else {
                arg.push(c);
            }
        }
        return Some(arg);
    }
}

pub fn runcmd(cmd: &str) -> AnyResult<String> {
    let argparser = ArgParser::new(cmd);
    let cmd: Vec<String> = argparser.into_iter().collect();
    let outbuf = match cmd.len() {
        0 => return Err(elog!("Empty command")),
        1 => Command::new(&cmd[0]).output(),
        _ => Command::new(&cmd[0]).args(&cmd[1..]).output(),
    };
    let outbuf = outbuf.context(elog!("failed to run command {:?}", cmd))?;
    if !outbuf.status.success() {
        let err = match std::str::from_utf8(&outbuf.stderr[..]) {
            Ok(e) => e.to_string(),
            Err(_) => format!("command exit with error: {:?}", outbuf.stderr),
        };
        return Err(elog!("{}", err));
    }
    let output = std::str::from_utf8(&outbuf.stdout[..])
        .context(elog!("failed to decode output: {:?}", outbuf.stdout))?;
    Ok(output.to_string())
}

#[test]
fn test_argparser() {
    let cmd = "git clone https://github.com/ikey4u/macddk '~/Library/Application Support/wikit/macddk'";
    let mut argparser = ArgParser::new(cmd);
    assert_eq!(Some("git".into()), argparser.next());
    assert_eq!(Some("clone".into()), argparser.next());
    assert_eq!(Some("https://github.com/ikey4u/macddk".into()), argparser.next());
    assert_eq!(Some("~/Library/Application Support/wikit/macddk".into()), argparser.next());
    assert_eq!(None, argparser.next());
}
