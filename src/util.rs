use crate::elog;
use crate::error::{AnyResult, Context};

use std::process::Command;
use std::collections::HashMap;

struct ArgParser<'a> {
    buf: &'a str,
    consumed: usize,
    total: usize,
}

impl<'a> ArgParser<'a> {
    fn new(buf: &'a str) -> Self {
        let buf = buf.trim();
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
        // Mark the ends of an argument parsing
        let mut spaced = false;
        for c in self.buf.chars().skip(self.consumed) {
            self.consumed += 1;
            if quote || c == '\'' || c == '\"' {
                if spaced {
                    self.consumed -= 1;
                    break;
                }
                // Take until the next same `c` or end
                if quote {
                    if c == quotech {
                        quote = false;
                    } else {
                        // Note that spaces in quote will also go here and keeped as they were
                        arg.push(c);
                    }
                } else {
                    quote = true;
                    quotech = c;
                }
            } else if c == ' ' {
                // Discard any non-quote repeted space
                if !spaced {
                    spaced = true;
                }
            } else {
                if spaced {
                    // All repeted spaces are consumed, and we have consumed once more non-spaced
                    // char, so we should end the parsing and go back one position. Notice that
                    // only non-quote space will go here, since all quote space will go into the
                    // first if branch.
                    self.consumed -= 1;
                    break;
                } else {
                    arg.push(c);
                }
            }
        }
        return Some(arg);
    }
}

pub fn runcmd(cmd: &str, envs: Option<Vec<(String, String)>>) -> AnyResult<String> {
    let argparser = ArgParser::new(cmd);
    let cmd: Vec<String> = argparser.into_iter().collect();
    let envs: HashMap<String, String> = if let Some(envs) = envs {
        envs.into_iter().collect()
    } else {
        HashMap::new()
    };
    let outbuf = match cmd.len() {
        0 => return Err(elog!("Empty command")),
        1 => Command::new(&cmd[0]).envs(&envs).output(),
        _ => Command::new(&cmd[0]).envs(&envs).args(&cmd[1..]).output(),
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
    let cmd = "a bc def";
    let mut argparser = ArgParser::new(cmd);
    assert_eq!(Some("a".into()), argparser.next());
    assert_eq!(Some("bc".into()), argparser.next());
    assert_eq!(Some("def".into()), argparser.next());

    let cmd = " a bc  def   ghi ";
    let mut argparser = ArgParser::new(cmd);
    assert_eq!(Some("a".into()), argparser.next());
    assert_eq!(Some("bc".into()), argparser.next());
    assert_eq!(Some("def".into()), argparser.next());
    assert_eq!(Some("ghi".into()), argparser.next());

    let cmd = " a bc 'def ghi' 'jkl  mno' 'pqr   st' ' x y z  ' ";
    let mut argparser = ArgParser::new(cmd);
    assert_eq!(Some("a".into()), argparser.next());
    assert_eq!(Some("bc".into()), argparser.next());
    assert_eq!(Some("def ghi".into()), argparser.next());
    assert_eq!(Some("jkl  mno".into()), argparser.next());
    assert_eq!(Some("pqr   st".into()), argparser.next());
    assert_eq!(Some(" x y z  ".into()), argparser.next());
}
