//! Wikit HTTP service. The router can also be embedded in another Axum application.
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use serde::Deserialize;
use std::{
    path::{Component, PathBuf},
    sync::Arc,
};
use wikit_base::Dictionary;
use wikit_proto::DictMeta;

struct Entry {
    id: String,
    dictionary: Arc<Dictionary>,
}
#[derive(Clone)]
pub struct Catalog(Arc<Vec<Entry>>);
impl Catalog {
    pub fn load(paths: &[PathBuf]) -> anyhow::Result<Self> {
        anyhow::ensure!(!paths.is_empty(), "Provide at least one --dictionary path");
        let mut entries = Vec::new();
        for path in paths {
            let dictionary = Dictionary::open(path)?;
            let id = wikit_base::crypto::md5(dictionary.path().to_string_lossy().as_bytes());
            if !entries.iter().any(|entry: &Entry| entry.id == id) {
                entries.push(Entry {
                    id,
                    dictionary: Arc::new(dictionary),
                });
            }
        }
        Ok(Self(Arc::new(entries)))
    }
    fn find(&self, id: &str) -> Result<Arc<Dictionary>, ApiError> {
        self.0
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.dictionary.clone())
            .ok_or((StatusCode::NOT_FOUND, "Unknown dictionary".into()))
    }
}
type ApiError = (StatusCode, String);
#[derive(Deserialize)]
struct Selection {
    dictname: String,
}
#[derive(Deserialize)]
struct Lookup {
    dictname: String,
    word: String,
}

pub fn router(catalog: Catalog) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/wikit/list", get(list))
        .route("/wikit/query", get(lookup))
        .route("/wikit/style", get(style))
        .route("/wikit/script", get(script))
        .route("/wikit/base", get(base))
        .route("/wikit/assets/{id}/{*path}", get(asset))
        .with_state(catalog)
}
async fn list(State(catalog): State<Catalog>) -> Json<Vec<DictMeta>> {
    Json(
        catalog
            .0
            .iter()
            .map(|e| DictMeta {
                id: e.id.clone(),
                name: e.dictionary.name().into(),
            })
            .collect(),
    )
}
async fn lookup(
    State(catalog): State<Catalog>,
    Query(query): Query<Lookup>,
) -> Result<Json<Vec<(String, String)>>, ApiError> {
    if query.word.trim().is_empty() || query.word.len() > 4096 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Enter a word between 1 and 4096 bytes".into(),
        ));
    }
    let dictionary = catalog.find(&query.dictname)?;
    tokio::task::spawn_blocking(move || dictionary.lookup(&query.word))
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Lookup worker failed".into(),
            )
        })?
        .map(Json)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cannot read dictionary entry".into(),
            )
        })
}
async fn style(
    State(catalog): State<Catalog>,
    Query(query): Query<Selection>,
) -> Result<impl IntoResponse, ApiError> {
    Ok((
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        catalog.find(&query.dictname)?.style().to_owned(),
    ))
}
async fn script(
    State(catalog): State<Catalog>,
    Query(query): Query<Selection>,
) -> Result<impl IntoResponse, ApiError> {
    Ok((
        [(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )],
        catalog.find(&query.dictname)?.script().to_owned(),
    ))
}
async fn base(
    State(catalog): State<Catalog>,
    Query(query): Query<Selection>,
) -> Result<String, ApiError> {
    catalog.find(&query.dictname)?;
    // Relative to the configured server base, including any reverse-proxy prefix.
    Ok(format!("wikit/assets/{}/", query.dictname))
}
async fn asset(
    State(catalog): State<Catalog>,
    Path((id, relative)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    let dictionary = catalog.find(&id)?;
    let relative = PathBuf::from(relative);
    if relative
        .components()
        .any(|c| !matches!(c, Component::Normal(s) if !s.to_string_lossy().starts_with('.')))
    {
        return Err((StatusCode::NOT_FOUND, "Asset not found".into()));
    }
    let root = dictionary.path().parent().unwrap().to_path_buf();
    let resolved = tokio::fs::canonicalize(root.join(relative))
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Asset not found".into()))?;
    if !resolved.starts_with(&root) {
        return Err((StatusCode::NOT_FOUND, "Asset not found".into()));
    }
    let extension = resolved
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if ![
        "css", "js", "png", "jpg", "jpeg", "gif", "svg", "webp", "ico", "woff", "woff2", "ttf",
        "otf", "mp3", "wav", "ogg", "mp4", "webm", "ini",
    ]
    .contains(&extension.as_str())
    {
        return Err((StatusCode::NOT_FOUND, "Asset not found".into()));
    }
    let bytes = tokio::fs::read(&resolved)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Asset not found".into()))?;
    let mime = mime_guess::from_path(&resolved)
        .first_or_octet_stream()
        .to_string();
    Ok((
        [
            (header::CONTENT_TYPE, mime),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".into()),
        ],
        bytes,
    ))
}
