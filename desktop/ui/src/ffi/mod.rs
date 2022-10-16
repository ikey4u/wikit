use wasm_bindgen::prelude::*;

pub type FFIResult<T> = anyhow::Result<T, JsValue>;

#[wasm_bindgen(module = "/src/ffi/ffi.js")]
extern "C" {
    #[wasm_bindgen(js_name = ffiHello, catch)]
    pub async fn ffi_hello(name: String) -> FFIResult<JsValue>;

    #[wasm_bindgen(js_name = startPreviewServer, catch)]
    pub async fn start_preview_server(dir: String) -> FFIResult<JsValue>;

    #[wasm_bindgen(js_name = stopPreviewServer, catch)]
    pub async fn stop_preview_server() -> FFIResult<JsValue>;

    #[wasm_bindgen(js_name = isPreviewServerUp, catch)]
    pub async fn is_preview_server_up() -> FFIResult<JsValue>;

    #[wasm_bindgen(js_name = open, catch)]
    pub async fn open() -> FFIResult<JsValue>;

    #[wasm_bindgen(js_name = lookUp, catch)]
    pub async fn lookup(dictid: String, word: String) -> FFIResult<JsValue>;

    #[wasm_bindgen(js_name = getDictList, catch)]
    pub async fn get_dict_list() -> FFIResult<JsValue>;
}
