use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/ffi/ffi.js")]
extern "C" {
    #[wasm_bindgen(js_name = ffiHello, catch)]
    pub async fn ffi_hello(name: String) -> Result<JsValue, JsValue>;
}
