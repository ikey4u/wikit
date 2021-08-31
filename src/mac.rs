use crate::error::{AnyResult, Context};
use crate::reader::MDXSource;
use std::path::Path;

fn ensure_mac_environment() -> AnyResult<()> {
    Ok(())
}

pub fn create_mac_dictionary<P>(mdxsrc: MDXSource, dictpath: P) -> AnyResult<()>
    where P: AsRef<Path>
{
    ensure_mac_environment()?;
    println!("dictpath: {}", dictpath.as_ref().display());
    for items in mdxsrc {
        println!("word: {}; meaning: {}", items.0, items.1);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::mac::create_mac_dictionary;

    #[test]
    fn test_create_mac_dictionary() {
        if let Some(mdxpat) = option_env!("TEST_MDX_FILE") {
        }
    }
}
