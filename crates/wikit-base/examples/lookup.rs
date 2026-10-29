use wikit_base::Dictionary;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args.next().ok_or("Usage: lookup DICTIONARY WORD")?;
    let word = args.next().ok_or("Usage: lookup DICTIONARY WORD")?;
    let dictionary = Dictionary::open(path)?;
    for (word, html) in dictionary.lookup(&word)? {
        println!("{word}\n{html}\n");
    }
    Ok(())
}
