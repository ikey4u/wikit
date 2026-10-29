use std::path::Path;
use wikit_base::{Dictionary, LocalDictionary};
#[test]
fn public_library_round_trip_and_resource_order() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("sample.txt");
    std::fs::write(&source, "hello\n<p>A greeting.</p>\n</>\n").unwrap();
    std::fs::write(
        dir.path().join("sample.toml"),
        "name='Example'\ncss=['first.css','second.css']\njs='first.js, second.js'\n",
    )
    .unwrap();
    for (file, text) in [
        ("first.css", "p{color:red}"),
        ("second.css", "p{color:blue}"),
        ("first.js", "var x = 1"),
        ("second.js", "x = 2"),
    ] {
        std::fs::write(dir.path().join(file), text).unwrap();
    }
    let path = LocalDictionary::create(&source, None::<&Path>).unwrap();
    let dictionary = Dictionary::open(path).unwrap();
    assert_eq!(dictionary.name(), "Example");
    assert_eq!(dictionary.lookup(" HELLO ").unwrap().len(), 1);
    assert!(dictionary.lookup("zzzzzzzzzz").unwrap().is_empty());
    assert!(dictionary.style().find("red").unwrap() < dictionary.style().find("blue").unwrap());
    assert!(dictionary.script().contains("\n;\n"));
    std::fs::remove_file(dir.path().join("second.css")).unwrap();
    assert!(LocalDictionary::create(&source, Some(dir.path().join("other.wikit"))).is_err());
}
#[test]
fn mdx_can_be_created_read_and_searched_via_public_api() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("sample.txt");
    let mdx = dir.path().join("sample.mdx");
    std::fs::write(
        &source,
        "hello\nA greeting.\n</>\nworld\nOur planet.\n</>\n",
    )
    .unwrap();
    wikit_base::mdict::create_mdx("Example", "Author", "Description", &source, &mdx).unwrap();
    let dictionary = Dictionary::open(mdx).unwrap();
    assert_eq!(dictionary.lookup("hello").unwrap()[0].0, "hello");
    assert_eq!(dictionary.lookup("wor").unwrap()[0].0, "world");
}
