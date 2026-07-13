use crate::error::{Result, WikitError};
use crate::reader::WikitSource;
use crate::util;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::{
    fs,
    path::{Path, PathBuf},
};

use axum::{
    body::Full,
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, TypedHeader,
    },
    handler::Handler,
    http::{Method, StatusCode, Uri},
    response::{self, Html, IntoResponse, Response},
    routing,
    routing::get_service,
    Router,
};
use notify::{DebouncedEvent, Watcher};
use std::net::SocketAddr;
use tokio::runtime::Builder;
use tokio::signal;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::Duration;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

#[derive(Debug)]
pub struct PreviewerState {
    db: rusqlite::Connection,
    wss: Option<WebSocket>,
}

pub struct Previewer {
    watchdir: PathBuf,
    svrdir: PathBuf,
    port: u16,
    state: Arc<Mutex<PreviewerState>>,
}

impl Previewer {
    pub fn new<P: AsRef<Path>>(watchdir: P) -> Result<Self> {
        let svrdir = tempfile::tempdir()?.into_path();
        let dbpath = svrdir.join("preview.db");
        let port = util::get_free_web_tcp_port(Some(8088))
            .ok_or_else(|| WikitError::new("failed to get preview server port"))?;
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
            port,
            state: Arc::new(Mutex::new(PreviewerState { db: db, wss: None })),
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run(self: Arc<Self>, shutdown_rx: Receiver<()>) -> Result<()> {
        let this = Arc::clone(&self);
        let mut consumer = tokio::spawn(async move { this.consumer(shutdown_rx).await });

        let this = Arc::clone(&self);
        let mut producer = tokio::spawn(async move { this.producer().await });

        tokio::select! {
            result = &mut consumer => {
                producer.abort();
                match result {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(e)) => Err(WikitError::new(format!("consumer exit with error: {e:?}"))),
                    Err(e) => Err(WikitError::new(format!("consumer task exit with error: {e:?}"))),
                }
            },
            result = &mut producer => {
                consumer.abort();
                match result {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(e)) => Err(WikitError::new(format!("producer exit with error: {e:?}"))),
                    Err(e) => Err(WikitError::new(format!("producer task exit with error: {e:?}"))),
                }
            },
        }
    }

    async fn index(Extension(state): Extension<Arc<Mutex<PreviewerState>>>) -> impl IntoResponse {
        let state = state.lock().await;
        let mut stmt = state
            .db
            .prepare("SELECT resource, size FROM preview where router = '/words'")
            .unwrap();
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
            "#
            .to_string()
        };
        Response::builder()
            .status(StatusCode::OK)
            .body(Full::from(page))
            .unwrap()
    }

    async fn resources(
        Extension(state): Extension<Arc<Mutex<PreviewerState>>>,
        uri: Uri,
    ) -> impl IntoResponse {
        (StatusCode::NOT_FOUND, format!("No route for: {uri}"))
    }

    // Write changes of dictionary source into database
    async fn producer(&self) -> Result<()> {
        fn update(wikit_source_dir: PathBuf, db: &rusqlite::Connection) -> Result<()> {
            let header_html = {
                let header_file = wikit_source_dir.join("header.wikit.txt");
                let mut mp = HashMap::new();
                for item in WikitSource::new(File::open(header_file)?) {
                    mp.insert(item.header.typ, item.body);
                }
                format!(
                    r#"
                    <head>
                        <script>
                        {}
                        </script>
                        <style>
                        {}
                        </style>
                    </head>
                "#,
                    mp.get("js").unwrap_or(&"".into()),
                    mp.get("css").unwrap_or(&"".into())
                )
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
                db.execute(
                    "INSERT OR REPLACE INTO preview (router, resource, size) VALUES (?1, ?2, ?3)",
                    rusqlite::params!["/words", html, html.len(),],
                )?;
            }

            Ok(())
        }

        // initialize updatding, do it in a block to avoid deadlock
        {
            let state = self.state.lock().await;
            update(self.watchdir.clone(), &state.db)?;
        }

        let (tx, rx) = channel();
        let mut watcher = notify::watcher(tx, Duration::from_millis(16)).unwrap();
        watcher
            .watch(self.watchdir.clone(), notify::RecursiveMode::Recursive)
            .unwrap();
        loop {
            // rx is not `Send`, we should own the received value
            let path = match rx.recv() {
                Ok(DebouncedEvent::Write(path)) => Some(path.to_owned()),
                _ => None,
            };
            // TODO(2022-04-25): filter out changed files
            if let Some(_) = path {
                let mut state = self.state.lock().await;
                update(self.watchdir.clone(), &state.db)?;
                if let Some(socket) = state.wss.as_mut() {
                    let _ = socket.send(Message::Text("CMD:RELOAD".into())).await;
                }
            }
        }
    }

    async fn wss(
        Extension(state): Extension<Arc<Mutex<PreviewerState>>>,
        ws: WebSocketUpgrade,
        user_agent: Option<TypedHeader<headers::UserAgent>>,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |mut socket: WebSocket| {
            async move {
                if let Some(Ok(Message::Text(msg))) = socket.recv().await {
                    match msg.as_str() {
                        "WIKIT_PREVIEWER_CONNECT" => {
                            if socket
                                .send(Message::Text("STATUS:CONNECTED".into()))
                                .await
                                .is_err()
                            {
                                println!("failed to send response to websocket client");
                            }
                            // initialize the websocket connection
                            let mut state = state.lock().await;
                            state.wss = Some(socket);
                        }
                        _ => {}
                    }
                }
            }
        })
    }

    // Listen and read database resource according to requested uri
    async fn consumer(&self, shutdown_rx: Receiver<()>) -> Result<()> {
        // TODO(2022-04-23): refine cors control
        let cors = CorsLayer::new()
            .allow_methods(vec![Method::GET, Method::POST])
            .allow_origin(Any);
        let app = Router::new()
            .route("/", routing::get(Self::index))
            .route("/wss", routing::get(Self::wss))
            .fallback(Self::resources.into_service())
            .layer(
                ServiceBuilder::new()
                    .layer(cors)
                    .layer(Extension(self.state.clone())),
            );
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let server = axum::Server::bind(&addr)
            .tcp_nodelay(true)
            .serve(app.into_make_service())
            .with_graceful_shutdown(self.shutdown(shutdown_rx));
        server
            .await
            .map_err(|e| WikitError::new(format!("{:?}", e)))
    }

    // Graceful shutdown: https://github.com/tokio-rs/axum/tree/main/examples/graceful-shutdown
    async fn shutdown(&self, mut shutdown_rx: Receiver<()>) {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
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
                println!(
                    "failed to remove svrdir: {} with error: {}",
                    self.svrdir.display(),
                    e
                );
            };
        }
    }
}
