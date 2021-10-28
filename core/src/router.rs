use sqlx::postgres::PgPoolOptions;
use rocket::{Build, Request, response::content, catch, get, catchers, routes};
use regex::Regex;
use once_cell::sync::Lazy;

static WIKIT_CONFIG: Lazy<crate::config::WikitConfig> = Lazy::new(|| {
    crate::config::load_config().expect("Cannot load wikit config")
});

#[catch(500)]
fn internal_error() -> &'static str {
    "It seems that you are not on earth"
}

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("I couldn't find '{}'. Try something else?", req.uri())
}

#[get("/q?<word>&<dictname>")]
async fn query(word: String, dictname: String) -> content::Html<String> {
    let meaning = if let Some(table) = &WIKIT_CONFIG.dict.get(&dictname) {
        if let Ok(pool) = PgPoolOptions::new().max_connections(500)
            .connect(&WIKIT_CONFIG.dburl).await {
            let sql = format!("SELECT word, meaning FROM {} WHERE word ilike $1", table);
            let query = sqlx::query_as::<_, (String, String)>(&sql)
                .bind(format!("{}", word))
                .fetch_one(&pool).await;
            match query {
                Ok(row) => {
                    row.1
                },
                Err(_e) => format!("<p>No such word: {}</p>", word)
            }
        } else {
            format!("<p>Database connection is broken</p>")
        }
    } else {
        format!("<p>Unknown dictionary: {}</p>", dictname)
    };
    // Remove built-in style link
    let re = Regex::new(r#"<link[^>]+.css[^>]+>"#).unwrap();
    let meaning = re.replace_all(meaning.as_str(), "");
    // Remote built-in image link
    let re = Regex::new(r#"<img[^>]+>"#).unwrap();
    let mut meaning = re.replace_all(&meaning, "").to_string();
    // Replace built-in `@@@LINK` to `entry://` format which is used by `ui` (`ui` will listen all
    // link click event, any link address that starts with `entry://` will cause a new remote
    // query).
    if meaning.starts_with("@@@LINK=") {
        let word = meaning.replace("@@@LINK=", "");
        let word = word.trim();
        meaning = format!("See <a href=\"entry://{}\" style=\"color: blue\">{}</a>", word, word);
    }
    content::Html(meaning.to_string())
}

pub fn rocket() -> rocket::Rocket<Build> {
    rocket::build()
        .mount("/dict/", routes![query])
        .register("/", catchers![internal_error, not_found])
}
