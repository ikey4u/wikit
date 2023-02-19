use std::os::raw::c_char;
use std::ffi::{CString, CStr};

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

#[no_mangle]
pub extern fn rust_greeting(to: *const c_char) -> *mut c_char {
    let c_str = unsafe { CStr::from_ptr(to) };
    let recipient = match c_str.to_str() {
        Err(_) => "there",
        Ok(string) => string,
    };

    CString::new(format!("Hello {}, welcome to rust word!", recipient)).unwrap().into_raw()
}

#[no_mangle]
pub unsafe extern fn Java_com_zhqli_wikit_ffi_RustBindings_greeting(env: JNIEnv, _: JClass, java_pattern: JString) -> jstring {
    // Our Java companion code might pass-in "world" as a string, hence the name.
    let world = rust_greeting(env.get_string(java_pattern).expect("invalid pattern string").as_ptr());
    // Retake pointer so that we can use it below and allow memory to be freed when it goes out of scope.
    let world_ptr = CString::from_raw(world);
    let output = env.new_string(world_ptr.to_str().unwrap()).expect("Couldn't create java string!");

    output.into_inner()
}


