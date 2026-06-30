#![allow(unexpected_cfgs)]

use anyhow::{anyhow, Context, Result as AnyResult};
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};
use serde_json::{json, Value};
use async_openai::Client;
use napi::{Error, Result as NapiResult};
use napi_derive::napi;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::mpsc::SyncSender;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use chrono::Utc;
use wikit_core::{config, crypto, preview, util, wikit};
use wikit_core::wikit::WikitDictionary;

static DICTDB: Lazy<Arc<Mutex<HashMap<String, WikitDictionary>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});
static INTERNAL_FS_PORT: AtomicU16 = AtomicU16::new(7561);
static STATIC_SERVER_STARTED: AtomicBool = AtomicBool::new(false);
static STATIC_SERVER_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
static PREVIEW_SERVER_STARTED: AtomicBool = AtomicBool::new(false);
static PREVIEW_SERVER_PORT: AtomicU16 = AtomicU16::new(0);
static PREVIEW_SHUTDOWN: Lazy<Mutex<Option<tokio::sync::broadcast::Sender<()>>>> = Lazy::new(|| Mutex::new(None));
static NATIVE_LOG_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

fn native_log(level: &str, module: &str, message: impl AsRef<str>) {
    let message = message.as_ref();
    let timestamp = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let line = format!("[{timestamp}] [{level}] [{module}] {message}\n");
    eprint!("{line}");
    let Ok(guard) = NATIVE_LOG_PATH.lock() else {
        return;
    };
    let Some(path) = guard.as_ref() else {
        return;
    };
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = file.write_all(line.as_bytes());
    }
}

#[napi]
pub fn init_native_logger(log_file_path: String) -> NapiResult<()> {
    let path = PathBuf::from(log_file_path.trim());
    if path.as_os_str().is_empty() {
        return Err(Error::from_reason("native log file path is empty"));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| napi_error("failed to create native log directory", e))?;
    }
    {
        let mut guard = NATIVE_LOG_PATH
            .lock()
            .map_err(|e| Error::from_reason(format!("failed to lock native logger state: {e}")))?;
        *guard = Some(path.clone());
    }
    native_log("INFO", "logger", format!("initialized at {}", path.display()));
    Ok(())
}

#[napi(object)]
pub struct DictMeta {
    pub name: String,
    pub id: String,
}

#[napi(object)]
pub struct LookupResponse {
    pub words: HashMap<String, String>,
    pub script: String,
    pub style: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct TranslationSettings {
    provider: String,
    endpoint: String,
    model: String,
    api_key: String,
    temperature: f32,
    timeout: u64,
}

impl Default for TranslationSettings {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            endpoint: "http://127.0.0.1:11434".to_string(),
            model: "qwen2.5:7b".to_string(),
            api_key: String::new(),
            temperature: 0.2,
            timeout: 60,
        }
    }
}

#[derive(Debug, Deserialize)]
struct TranslationRequest {
    text: String,
    source: String,
    target: String,
}

#[derive(Debug, Serialize)]
struct TranslationResponse {
    text: String,
    provider: String,
    model: String,
}

#[derive(Debug, Serialize)]
struct TranslationTestResponse {
    ok: bool,
    message: String,
}

fn napi_error<E: std::fmt::Debug>(context: &str, error: E) -> Error {
    Error::from_reason(format!("{context}: {error:?}"))
}

fn lock_dictdb() -> NapiResult<std::sync::MutexGuard<'static, HashMap<String, WikitDictionary>>> {
    DICTDB.lock().map_err(|e| Error::from_reason(format!("failed to lock dictionary database: {e}")))
}

fn lock_preview_shutdown() -> NapiResult<std::sync::MutexGuard<'static, Option<tokio::sync::broadcast::Sender<()>>>> {
    PREVIEW_SHUTDOWN.lock().map_err(|e| Error::from_reason(format!("failed to lock preview server state: {e}")))
}

fn translation_config_path() -> AnyResult<PathBuf> {
    Ok(config::get_config_dir()?.join("translation.toml"))
}

fn sanitize_translation_settings(mut settings: TranslationSettings) -> TranslationSettings {
    settings.provider = settings.provider.trim().to_string();
    settings.endpoint = settings.endpoint.trim().trim_end_matches('/').to_string();
    settings.model = settings.model.trim().to_string();
    settings.api_key = settings.api_key.trim().to_string();
    if settings.provider.is_empty() {
        settings.provider = TranslationSettings::default().provider;
    }
    if settings.provider == "deepseek" {
        if settings.endpoint == "https://api.deepseek.com/v1" {
            settings.endpoint = "https://api.deepseek.com".to_string();
        }
        if settings.model == "deepseek-chat"
            || (settings.model != "deepseek-v4-flash" && settings.model != "deepseek-v4-pro")
        {
            settings.model = "deepseek-v4-pro".to_string();
        }
    }
    if settings.endpoint.is_empty() {
        settings.endpoint = if settings.provider == "deepseek" {
            "https://api.deepseek.com".to_string()
        } else {
            TranslationSettings::default().endpoint
        };
    }
    if settings.model.is_empty() {
        settings.model = if settings.provider == "deepseek" {
            "deepseek-v4-pro".to_string()
        } else {
            TranslationSettings::default().model
        };
    }
    if !(0.0..=2.0).contains(&settings.temperature) {
        settings.temperature = TranslationSettings::default().temperature;
    }
    settings.timeout = settings.timeout.clamp(5, 300);
    settings
}

fn load_translation_settings_inner() -> AnyResult<TranslationSettings> {
    let path = translation_config_path()?;
    if !path.exists() {
        let settings = TranslationSettings::default();
        let content = toml::to_string_pretty(&settings)?;
        File::create(&path)?.write_all(content.as_bytes())?;
        return Ok(settings);
    }

    let mut content = String::new();
    File::open(&path)?.read_to_string(&mut content)?;
    if content.trim().is_empty() {
        return Ok(TranslationSettings::default());
    }
    Ok(sanitize_translation_settings(toml::from_str(&content)?))
}

fn save_translation_settings_inner(settings: TranslationSettings) -> AnyResult<TranslationSettings> {
    let settings = sanitize_translation_settings(settings);
    let path = translation_config_path()?;
    let content = toml::to_string_pretty(&settings)?;
    File::create(path)?.write_all(content.as_bytes())?;
    Ok(settings)
}

fn normalized_api_base(settings: &TranslationSettings) -> String {
    let endpoint = settings.endpoint.trim().trim_end_matches('/');
    match settings.provider.as_str() {
        "ollama" | "llama-cpp" | "vllm" => {
            if endpoint.ends_with("/v1") {
                endpoint.to_string()
            } else {
                format!("{endpoint}/v1")
            }
        }
        "deepseek" => endpoint.trim_end_matches("/v1").to_string(),
        _ => endpoint.to_string(),
    }
}

fn language_name(code: &str) -> &str {
    match code {
        "auto" => "the automatically detected source language",
        "zh" => "Chinese",
        "en" => "English",
        "ja" => "Japanese",
        "ko" => "Korean",
        "fr" => "French",
        "de" => "German",
        "es" => "Spanish",
        "ru" => "Russian",
        _ => code,
    }
}

async fn translate_with_settings(settings: TranslationSettings, request: TranslationRequest) -> AnyResult<TranslationResponse> {
    let text = request.text.trim();
    if text.is_empty() {
        return Err(anyhow!("translation input is empty"));
    }

    let api_key = if settings.api_key.is_empty() {
        "not-needed"
    } else {
        settings.api_key.as_str()
    };
    let config = OpenAIConfig::new()
        .with_api_base(normalized_api_base(&settings))
        .with_api_key(api_key);
    let client = Client::with_config(config);
    let source = language_name(&request.source);
    let target = language_name(&request.target);
    let system_prompt = format!(
        "You are a professional translation engine. Translate faithfully from {source} to {target}. Preserve meaning, tone, formatting, numbers, URLs, code blocks, and proper nouns. Output only the translation."
    );
    let user_prompt = format!("Text:\n{text}");

    if settings.provider == "deepseek" {
        let model = settings.model.clone();
        let mut request = json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_prompt}
            ],
            "stream": false,
            "temperature": settings.temperature
        });
        if request["model"] == "deepseek-v4-pro" {
            request["reasoning_effort"] = json!("high");
            request["thinking"] = json!({"type": "enabled"});
        }
        let timeout = Duration::from_secs(settings.timeout);
        let response: Value = tokio::time::timeout(timeout, client.chat().create_byot(request))
            .await
            .map_err(|_| anyhow!("translation request timed out"))??;
        let translated = response
            .get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if translated.is_empty() {
            return Err(anyhow!("model returned empty translation"));
        }
        return Ok(TranslationResponse {
            text: translated,
            provider: settings.provider,
            model: settings.model,
        });
    }

    let messages = vec![
        ChatCompletionRequestSystemMessageArgs::default()
            .content(system_prompt)
            .build()?
            .into(),
        ChatCompletionRequestUserMessageArgs::default()
            .content(user_prompt)
            .build()?
            .into(),
    ];
    let chat_request = CreateChatCompletionRequestArgs::default()
        .model(settings.model.clone())
        .messages(messages)
        .temperature(settings.temperature)
        .build()?;
    let timeout = Duration::from_secs(settings.timeout);
    let response = tokio::time::timeout(timeout, client.chat().create(chat_request))
        .await
        .map_err(|_| anyhow!("translation request timed out"))??;
    let translated = response
        .choices
        .first()
        .and_then(|choice| choice.message.content.clone())
        .unwrap_or_default()
        .trim()
        .to_string();
    if translated.is_empty() {
        return Err(anyhow!("model returned empty translation"));
    }

    Ok(TranslationResponse {
        text: translated,
        provider: settings.provider,
        model: settings.model,
    })
}

fn try_bind_static_port(port: u16) -> Option<TcpListener> {
    if util::is_chromium_restricted_port(port) {
        return None;
    }
    TcpListener::bind(("127.0.0.1", port)).ok()
}

fn bind_static_server_port() -> NapiResult<(u16, TcpListener)> {
    let preferred = INTERNAL_FS_PORT.load(Ordering::SeqCst);
    if let Some(listener) = try_bind_static_port(preferred) {
        native_log("INFO", "static-server", format!("using preferred port {preferred}"));
        return Ok((preferred, listener));
    }

    for port in 6000..9000 {
        if let Some(listener) = try_bind_static_port(port) {
            native_log(
                "INFO",
                "static-server",
                format!("preferred port {preferred} unavailable, using {port}"),
            );
            return Ok((port, listener));
        }
    }

    Err(Error::from_reason("failed to bind static file server port"))
}

fn run_static_file_server(listener: TcpListener, ready_tx: SyncSender<()>) -> AnyResult<()> {
    use axum::{http::StatusCode, routing::get_service, Router};
    use tower_http::services::ServeDir;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(5)
        .enable_all()
        .build()?;

    rt.block_on(async move {
        let app = Router::new().nest(
            "/static",
            get_service(ServeDir::new(config::get_static_dir()?)).handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {error}"),
                )
            }),
        );
        let server = axum::Server::from_tcp(listener)?;
        ready_tx
            .send(())
            .map_err(|_| anyhow!("static file server startup cancelled"))?;
        server.serve(app.into_make_service()).await?;
        Ok(())
    })
}

fn ensure_static_file_server() -> NapiResult<u16> {
    let _guard = STATIC_SERVER_LOCK
        .lock()
        .map_err(|e| Error::from_reason(format!("failed to lock static file server state: {e}")))?;

    if STATIC_SERVER_STARTED.load(Ordering::SeqCst) {
        let port = INTERNAL_FS_PORT.load(Ordering::SeqCst);
        native_log("INFO", "static-server", format!("already running on port {port}"));
        return Ok(port);
    }

    let (port, listener) = bind_static_server_port()?;
    INTERNAL_FS_PORT.store(port, Ordering::SeqCst);
    STATIC_SERVER_STARTED.store(true, Ordering::SeqCst);
    native_log("INFO", "static-server", format!("starting background thread on port {port}"));

    let (ready_tx, ready_rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        if let Err(error) = run_static_file_server(listener, ready_tx) {
            native_log(
                "ERROR",
                "static-server",
                format!("failed on port {port}: {error:?}"),
            );
            STATIC_SERVER_STARTED.store(false, Ordering::SeqCst);
        }
    });

    match ready_rx.recv_timeout(Duration::from_secs(10)) {
        Ok(()) => Ok(port),
        Err(error) => {
            STATIC_SERVER_STARTED.store(false, Ordering::SeqCst);
            Err(Error::from_reason(format!(
                "static file server startup timed out on port {port}: {error}"
            )))
        }
    }
}

fn write_file_once(content: &[u8], file: &Path) -> NapiResult<()> {
    if !file.exists() {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file)
            .map_err(|e| napi_error("failed to open static resource", e))?;
        file.write_all(content)
            .map_err(|e| napi_error("failed to write static resource", e))?;
    }
    Ok(())
}

#[napi]
pub fn start_static_file_server() -> NapiResult<u16> {
    ensure_static_file_server()
}

#[napi]
pub fn get_dict_list() -> NapiResult<Vec<DictMeta>> {
    let mut dictlist = Vec::new();
    let mut dictdb = lock_dictdb()?;
    dictdb.clear();

    if let Ok(dicts) = wikit::load_client_dictionary() {
        for dict in dicts {
            match &dict {
                WikitDictionary::Local(ld) => {
                    let id = ld.path.display().to_string();
                    dictlist.push(DictMeta {
                        name: ld.head.name.clone(),
                        id: id.clone(),
                    });
                    dictdb.insert(id, dict.clone());
                }
                WikitDictionary::Remote(rd) => {
                    if let Ok(metas) = rd.get_dict_list() {
                        for meta in metas {
                            dictdb.insert(meta.id.clone(), dict.clone());
                            dictlist.push(DictMeta {
                                name: meta.name,
                                id: meta.id,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(dictlist)
}

#[napi]
pub fn load_local_dictionary(path: String) -> NapiResult<DictMeta> {
    let source_path = PathBuf::from(&path);
    if !source_path.exists() {
        return Err(Error::from_reason(format!("dictionary file not found: {path}")));
    }

    let suffix = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    let wikit_path = match suffix.as_str() {
        "wikit" => source_path.clone(),
        "mdx" => {
            let cache_dir = config::get_config_dir()
                .map_err(|e| napi_error("failed to get config directory", e))?
                .join("local-dictionaries");
            fs::create_dir_all(&cache_dir)
                .map_err(|e| napi_error("failed to create local dictionary cache", e))?;
            let stem = source_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("dictionary");
            let hash = crypto::md5(path.as_bytes());
            let out_path = cache_dir.join(format!("{stem}-{hash}.wikit"));
            wikit::LocalDictionary::create(&source_path, Some(&out_path))
                .map_err(|e| napi_error("failed to convert mdx dictionary", e))?
        }
        _ => {
            return Err(Error::from_reason(format!("unsupported dictionary type: {suffix}")));
        }
    };

    let local = wikit::LocalDictionary::load(&wikit_path)
        .map_err(|e| napi_error("failed to load local dictionary", e))?;
    let id = local.path.display().to_string();
    let name = local.head.name.clone();
    config::register_client_dictionary_uri(&wikit_path)
        .map_err(|e| napi_error("failed to register dictionary in config", e))?;

    let mut dictdb = lock_dictdb()?;
    dictdb.insert(id.clone(), WikitDictionary::Local(local));
    Ok(DictMeta { name, id })
}

#[napi]
pub fn lookup(dictid: String, word: String) -> NapiResult<LookupResponse> {
    let mut words = HashMap::new();
    let mut script = String::new();
    let mut style = String::new();
    let dictdb = lock_dictdb()?;

    if let Some(dict) = dictdb.get(&dictid) {
        match dict {
            WikitDictionary::Local(ld) => {
                if let Ok(entries) = ld.lookup(&word) {
                    for (key, value) in entries {
                        words.insert(key, value);
                    }
                }
                script.push_str(ld.get_script());
                style.push_str(ld.get_style());
            }
            WikitDictionary::Remote(rd) => {
                if let Ok(entries) = rd.lookup(&word, &dictid) {
                    for (key, value) in entries {
                        words.insert(key, value);
                    }
                }
                script.push_str(&rd.get_script(&dictid));
                style.push_str(&rd.get_style(&dictid));
            }
        }
    }

    let staticdir = config::get_static_dir().map_err(|e| napi_error("failed to get static directory", e))?;
    let staticid = crypto::md5(dictid.as_bytes());
    let cssfile = staticdir.join(format!("{staticid}.css"));
    let jsfile = staticdir.join(format!("{staticid}.js"));
    write_file_once(style.as_bytes(), cssfile.as_path())?;
    write_file_once(script.as_bytes(), jsfile.as_path())?;
    if let Some(meaning) = words.get(&word) {
        let wordfile = staticdir.join(format!("{staticid}_{word}.html"));
        write_file_once(meaning.as_bytes(), wordfile.as_path())?;
    }

    let port = ensure_static_file_server()?;
    let style = format!(r#" <link rel="stylesheet" href="http://127.0.0.1:{port}/static/{staticid}.css"> "#);
    let script = format!(r#" <script type="text/javascript" src="http://127.0.0.1:{port}/static/{staticid}.js"></script> "#);

    Ok(LookupResponse { words, script, style })
}

#[napi]
pub fn get_translation_settings() -> NapiResult<String> {
    let settings = load_translation_settings_inner()
        .map_err(|e| napi_error("failed to load translation settings", e))?;
    serde_json::to_string(&settings)
        .map_err(|e| napi_error("failed to serialize translation settings", e))
}

#[napi]
pub fn save_translation_settings(settings_json: String) -> NapiResult<String> {
    let settings = serde_json::from_str::<TranslationSettings>(&settings_json)
        .map_err(|e| napi_error("failed to parse translation settings", e))?;
    let settings = save_translation_settings_inner(settings)
        .map_err(|e| napi_error("failed to save translation settings", e))?;
    serde_json::to_string(&settings)
        .map_err(|e| napi_error("failed to serialize translation settings", e))
}

#[napi]
pub async fn translate_text(request_json: String) -> NapiResult<String> {
    let request = serde_json::from_str::<TranslationRequest>(&request_json)
        .map_err(|e| napi_error("failed to parse translation request", e))?;
    let settings = load_translation_settings_inner()
        .map_err(|e| napi_error("failed to load translation settings", e))?;
    let response = translate_with_settings(settings, request)
        .await
        .map_err(|e| napi_error("translation failed", e))?;
    serde_json::to_string(&response)
        .map_err(|e| napi_error("failed to serialize translation response", e))
}

#[napi]
pub async fn test_translation_connection(settings_json: String) -> NapiResult<String> {
    let settings = serde_json::from_str::<TranslationSettings>(&settings_json)
        .map_err(|e| napi_error("failed to parse translation settings", e))?;
    let request = TranslationRequest {
        text: "hello".to_string(),
        source: "en".to_string(),
        target: "zh".to_string(),
    };
    let result = translate_with_settings(sanitize_translation_settings(settings), request).await;
    let response = match result {
        Ok(_) => TranslationTestResponse {
            ok: true,
            message: "连接成功".to_string(),
        },
        Err(error) => TranslationTestResponse {
            ok: false,
            message: format!("连接失败: {error}"),
        },
    };
    serde_json::to_string(&response)
        .map_err(|e| napi_error("failed to serialize test response", e))
}

#[napi]
pub fn ffi_hello(name: String) -> NapiResult<String> {
    if name.is_empty() {
        Err(Error::from_reason("name is empty"))
    } else {
        Ok(format!("ffi_hello got name: {name}"))
    }
}

#[napi]
pub fn start_preview_server(dir: String) -> NapiResult<u16> {
    if PREVIEW_SERVER_STARTED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        let port = PREVIEW_SERVER_PORT.load(Ordering::SeqCst);
        native_log("INFO", "preview-server", format!("already running on port {port}"));
        return Ok(port);
    }

    native_log(
        "INFO",
        "preview-server",
        format!(
            "creating previewer for dir '{}'",
            if dir.is_empty() { "<empty>" } else { dir.as_str() }
        ),
    );
    let previewer = match preview::Previewer::new(dir) {
        Ok(previewer) => previewer,
        Err(error) => {
            PREVIEW_SERVER_STARTED.store(false, Ordering::SeqCst);
            PREVIEW_SERVER_PORT.store(0, Ordering::SeqCst);
            native_log("ERROR", "preview-server", format!("failed to create previewer: {error:?}"));
            return Err(napi_error("failed to create preview server", error));
        }
    };
    let port = previewer.port();
    PREVIEW_SERVER_PORT.store(port, Ordering::SeqCst);
    native_log("INFO", "preview-server", format!("listening on port {port}"));

    let (tx, rx) = tokio::sync::broadcast::channel(1);
    {
        let mut shutdown = lock_preview_shutdown()?;
        *shutdown = Some(tx);
    }

    std::thread::spawn(move || {
        let result = || -> AnyResult<()> {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(5)
                .enable_all()
                .build()?;
            rt.block_on(async move {
                Arc::new(previewer).run(rx).await.context("preview server exited with error")
            })
        }();

        if let Err(error) = result {
            native_log("ERROR", "preview-server", format!("exited with error: {error:?}"));
        }
        PREVIEW_SERVER_STARTED.store(false, Ordering::SeqCst);
        PREVIEW_SERVER_PORT.store(0, Ordering::SeqCst);
        if let Ok(mut shutdown) = PREVIEW_SHUTDOWN.lock() {
            *shutdown = None;
        }
    });

    Ok(port)
}

#[napi]
pub fn stop_preview_server() -> NapiResult<()> {
    native_log("INFO", "preview-server", "stopping");
    if let Some(sender) = lock_preview_shutdown()?.as_ref() {
        let _ = sender.send(());
    }
    PREVIEW_SERVER_STARTED.store(false, Ordering::SeqCst);
    PREVIEW_SERVER_PORT.store(0, Ordering::SeqCst);
    Ok(())
}

#[napi]
pub fn is_preview_server_up() -> NapiResult<bool> {
    Ok(PREVIEW_SERVER_STARTED.load(Ordering::SeqCst))
}

#[napi]
pub fn get_config_dir() -> NapiResult<String> {
    config::get_config_dir()
        .map(|path| path.display().to_string())
        .map_err(|e| napi_error("failed to get config directory", e))
}

#[napi(object)]
pub struct DictInfo {
    pub id: String,
    pub name: String,
    pub desc: String,
    pub script: String,
    pub style: String,
}

#[napi(object)]
pub struct SearchEntry {
    pub word: String,
    pub definition: String,
}

#[napi]
pub fn get_dict_info(dictid: String) -> NapiResult<DictInfo> {
    let dictdb = lock_dictdb()?;
    if let Some(dict) = dictdb.get(&dictid) {
        match dict {
            WikitDictionary::Local(ld) => Ok(DictInfo {
                id: dictid,
                name: ld.head.name.clone(),
                desc: ld.head.desc.clone(),
                script: ld.head.script.clone(),
                style: ld.head.style.clone(),
            }),
            WikitDictionary::Remote(rd) => Ok(DictInfo {
                id: dictid.clone(),
                name: dictid.clone(),
                desc: String::new(),
                script: rd.get_script(&dictid),
                style: rd.get_style(&dictid),
            }),
        }
    } else {
        Err(Error::from_reason(format!("dictionary not found: {dictid}")))
    }
}

#[napi]
pub fn search_dict(dictid: String, word: String) -> NapiResult<Vec<SearchEntry>> {
    let dictdb = lock_dictdb()?;
    let mut results = Vec::new();
    if let Some(dict) = dictdb.get(&dictid) {
        let entries = match dict {
            WikitDictionary::Local(ld) => ld.lookup(&word),
            WikitDictionary::Remote(rd) => rd.lookup(&word, &dictid),
        };
        if let Ok(entries) = entries {
            let mut seen = HashSet::new();
            for (w, def) in entries {
                if w.is_empty() || !seen.insert(w.clone()) {
                    continue;
                }
                results.push(SearchEntry { word: w, definition: def });
            }
        }
    }
    Ok(results)
}

use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};

#[napi]
pub async fn build_dictionary(
    srcfile: String,
    outfile: String,
    progress_callback: ThreadsafeFunction<f64>,
) -> NapiResult<String> {
    tokio::task::spawn_blocking(move || -> NapiResult<String> {
        let srcfile_path = PathBuf::from(&srcfile);
        let outfile_path = PathBuf::from(&outfile);

        if !srcfile_path.exists() {
            return Err(Error::from_reason(format!("source file not found: {srcfile}")));
        }

        let suffix = srcfile_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        if suffix != "txt" && suffix != "mdx" {
            return Err(Error::from_reason(format!("unsupported source type: {}", suffix)));
        }

        let mut report_progress = |value: f64| {
            progress_callback.call(
                Ok(value.clamp(0.0, 1.0)),
                ThreadsafeFunctionCallMode::Blocking,
            );
        };
        report_progress(0.0);

        let result = wikit::LocalDictionary::create_with_progress(
            &srcfile_path,
            Some(&outfile_path),
            &mut report_progress,
        );

        match result {
            Ok(path) => {
                report_progress(1.0);
                Ok(serde_json::json!({"ok": true, "output": path.display().to_string()}).to_string())
            }
            Err(e) => {
                report_progress(1.0);
                Ok(serde_json::json!({"ok": false, "error": e.to_string()}).to_string())
            }
        }
    })
    .await
    .map_err(|e| napi_error("failed to join dictionary build task", e))?
}
