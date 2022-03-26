const invoke = window.__TAURI__.invoke

export async function ffiHello(name) {
    return await invoke("ffi_hello", { name: name });
}
