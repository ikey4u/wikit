use crate::error::{Result, WikitError};
use crate::reader::WikitSource;

use std::cell::{Cell, RefCell};
use std::{path::{Path, PathBuf}, fs};
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::fs::File;
use std::collections::HashMap;

use axum::{
    body::Full,
    response::{self, Html, Response, IntoResponse},
    http::{StatusCode, Uri},
    routing::get_service,
    Router,
    routing,
    handler::Handler,
    extract::Extension,
};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tokio::runtime::Builder;
use tokio::signal;
use tokio::sync::broadcast::{self, Sender, Receiver};
use tokio::sync::mpsc;
use tokio::time::Duration;
use notify::{Watcher, DebouncedEvent};

pub struct Previewer {
    watchdir: PathBuf,
    svrdir: PathBuf,
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl Previewer {
    pub fn new<P: AsRef<Path>>(watchdir: P) -> Result<Self> {
        let svrdir = tempfile::tempdir()?.into_path();
        let dbpath = svrdir.join("preview.db");
        let db = rusqlite::Connection::open(&dbpath)?;
        db.execute(
            "CREATE TABLE IF NOT EXISTS preview (
            router TEXT PRIMARY KEY,
            resource BLOB NOT NULL,
            size INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(Self {
            watchdir: watchdir.as_ref().into(),
            svrdir,
            db: Arc::new(Mutex::new(db)),
        })
    }

    pub async fn run(self: Arc<Self>, shutdown_rx: Receiver<()>) -> Result<()> {
        let this = Arc::clone(&self);
        let wathcer = tokio::spawn(async move {
            if let Err(e) = this.consumer(shutdown_rx).await {
                return Err(e);
            }
            Ok(())
        });

        let this = Arc::clone(&self);
        let apisrv = tokio::spawn(async move {
            if let Err(e) = this.producer().await {
                return Err(e);
            }
            Ok(())
        });

        tokio::select!{
            Ok(Err(e)) = wathcer => {
                return Err(WikitError::new(format!("watcher exit with error: {e:?}")))
            },
            Ok(Err(e)) = apisrv => {
                return Err(WikitError::new(format!("apisrv exit with error: {e:?}")));
            },
        };
    }

    async fn index(Extension(db): Extension<Arc<Mutex<rusqlite::Connection>>>) -> impl IntoResponse {
        let db = db.lock().unwrap();
        let mut stmt = db.prepare(
            "SELECT resource, size FROM preview where router = '/words'",
        ).unwrap();
        let mut rows = stmt.query([]).unwrap();
        let page = if let Ok(Some(row)) = rows.next() {
            let page: String = row.get(0).unwrap();
            page
        } else {
            r#"
            <html>
                <head>
                    <script>
                        setTimeout(function() {
                           window.location.reload(1);
                        }, 1600);
                    </script>
                </head>
                <body>
                    No preview page is found
                </body>
            </html>
            "#.to_string()
        };
        Response::builder()
            .status(StatusCode::OK)
            .body(Full::from(page))
            .unwrap()
    }

    async fn resources(Extension(db): Extension<Arc<Mutex<rusqlite::Connection>>>, uri: Uri) -> impl IntoResponse {
        (StatusCode::NOT_FOUND, format!("No route for: {uri}"))
    }

    // Write changes of dictionary source into database
    async fn producer(&self) -> Result<()> {
        fn update(wikit_source_dir: PathBuf, db: Arc<Mutex<rusqlite::Connection>>) -> Result<()> {
            let header_html = {
                let header_file = wikit_source_dir.join("header.wikit.txt");
                let mut mp = HashMap::new();
                for item in WikitSource::new(File::open(header_file)?) {
                    mp.insert(item.header.typ, item.body);
                }
                format!(r#"
                    <head>
                        <script>
                        {}
                        </script>
                        <style>
                        {}
                        </style>
                    </head>
                "#, mp.get("js").unwrap_or(&"".into()), mp.get("css").unwrap_or(&"".into()))
            };
            let body_html = {
                let mut html = "<body>".to_string();

                let mut wordcnt = 0;
                let preview_file = wikit_source_dir.join("preview.wikit.txt");
                for item in WikitSource::new(File::open(preview_file)?) {
                    if wordcnt > 100 {
                        break;
                    }
                    if item.header.typ == "word" {
                        html += item.body.as_str();
                        wordcnt += 1;
                    }
                }

                html += "</body>";
                html
            };

            let html = format!("<html>{}{}</html>", header_html, body_html);
            {
                // TODO(2022-04-10): remove unwrap
                db.lock().unwrap().execute(
                    "INSERT OR REPLACE INTO preview (router, resource, size) VALUES (?1, ?2, ?3)",
                    rusqlite::params![
                        "/words",
                        html,
                        html.len(),
                    ],
                )?;
            }

            Ok(())
        }

        update(self.watchdir.clone(), self.db.clone())?;
        let (tx, rx) = channel();
        let mut watcher = notify::watcher(tx, Duration::from_millis(16)).unwrap();
        watcher.watch(self.watchdir.clone(), notify::RecursiveMode::Recursive).unwrap();
        loop {
            match rx.recv() {
                Ok(DebouncedEvent::Write(path)) => {
                    println!("write file: {}", path.display());
                    update(self.watchdir.clone(), self.db.clone())?;
                }
                Ok(_) => {}
                Err(e) => {
                    println!("watch error: {:?}", e);
                }
            }
        }
    }

    // Listen and read database resource according to requested uri
    async fn consumer(&self, shutdown_rx: Receiver<()>) -> Result<()> {
        let app = Router::new()
            .route("/", routing::get(Self::index))
            .fallback(Self::resources.into_service())
            .layer(Extension(self.db.clone()));
        let addr = SocketAddr::from(([127, 0, 0, 1], 8088));
        let server = axum::Server::bind(&addr)
            .tcp_nodelay(true)
            .serve(app.into_make_service())
            .with_graceful_shutdown(self.shutdown(shutdown_rx));
        server.await.map_err(|e| WikitError::new(format!("{:?}", e)))
    }

    // Graceful shutdown: https://github.com/tokio-rs/axum/tree/main/examples/graceful-shutdown
    async fn shutdown(&self, mut shutdown_rx: Receiver<()>) {
        let ctrl_c = async {
            signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler").recv().await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
            _ = shutdown_rx.recv() => {},
        };
    }
}

impl Drop for Previewer {
    fn drop(&mut self) {
        if self.svrdir.exists() {
            if let Err(e) = fs::remove_dir_all(&self.svrdir) {
                println!("failed to remove svrdir: {} with error: {}", self.svrdir.display(), e);
            };
        }
    }
}
