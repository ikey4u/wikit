use regex::Regex;

// normalize is used to convert html to xhtml, this is not a complete implementation for now,
// it grows case by case when creating mac dictionary from html extracted from mdx file.
pub fn normalize(html: &str) -> String {
    let html = html.replace("&", "&amp;");
    let html = if let Ok(re) = Regex::new(r"<link(?P<content>[^>]+)>") {
        re.replace(&html, "<link ${content} />").to_string()
    } else {
        html
    };
    return html;
}
