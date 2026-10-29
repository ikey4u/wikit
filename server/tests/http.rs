use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn dictionary_endpoints_and_assets() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("sample.txt");
    std::fs::write(
        &source,
        "hello\n<p>Hello world</p>\n</>\n你好\n<p>中文释义</p>\n</>\n",
    )
    .unwrap();
    std::fs::write(
        dir.path().join("sample.toml"),
        "name = 'Sample'\ncss = ['a.css', 'b.css']\njs = 'a.js, b.js'\n",
    )
    .unwrap();
    for (name, content) in [
        ("a.css", "p{color:red}"),
        ("b.css", "p{color:blue}"),
        ("a.js", "var x=1"),
        ("b.js", "x=2"),
        ("secret.toml", "secret"),
    ] {
        std::fs::write(dir.path().join(name), content).unwrap();
    }
    let path = wikit_base::LocalDictionary::create(&source, None::<&std::path::Path>).unwrap();
    let catalog = wikit_server::Catalog::load(&[path.clone()]).unwrap();
    let app = wikit_server::router(catalog);
    let id = wikit_base::crypto::md5(path.canonicalize().unwrap().to_string_lossy().as_bytes());
    // Direct query works before /list is ever called.
    let query = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/wikit/query?dictname={id}&word=hello"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(query.status(), StatusCode::OK);
    let body = query.into_body().collect().await.unwrap().to_bytes();
    let entries: Vec<(String, String)> = serde_json::from_slice(&body).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, "hello");
    for (uri, status, contains) in [
        ("/wikit/list".into(), 200, "Sample"),
        (format!("/wikit/style?dictname={id}"), 200, "color:blue"),
        (format!("/wikit/script?dictname={id}"), 200, ";"),
        (format!("/wikit/base?dictname={id}"), 200, "wikit/assets/"),
        (
            format!("/wikit/query?dictname={id}&word=%E4%BD%A0%E5%A5%BD"),
            200,
            "中文释义",
        ),
        (
            format!("/wikit/query?dictname={id}&word=zzzzzzzzzz"),
            200,
            "[]",
        ),
        (
            "/wikit/query?dictname=absent&word=hello".into(),
            404,
            "Unknown",
        ),
        (
            format!("/wikit/query?dictname={id}&word="),
            400,
            "Enter a word",
        ),
        (format!("/wikit/assets/{id}/a.css"), 200, "color:red"),
        (
            format!("/wikit/assets/{id}/secret.toml"),
            404,
            "Asset not found",
        ),
        (
            format!("/wikit/assets/{id}/%2e%2e/Cargo.toml"),
            404,
            "Asset not found",
        ),
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), status);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(String::from_utf8_lossy(&body).contains(contains));
    }
}
