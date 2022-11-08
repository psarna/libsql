use std::ffi::{c_char, c_uchar, c_void, CString};
use wasmtime::{Config, Engine, Module};

#[no_mangle]
pub fn libsql_compile_wasm_module(
    engine: *const wasmtime::Engine,
    p_src_body: *const c_uchar,
    n_body: i32,
    err_msg_buf: *mut *const c_char,
) -> *const c_void {
    let src_body: &[u8] = unsafe { std::slice::from_raw_parts(p_src_body, n_body as usize) };

    let source_already_compiled =
        n_body >= 4 && &src_body[..4] == &['\0' as u8, 'a' as u8, 's' as u8, 'm' as u8];

    let compiled_owned = if source_already_compiled {
        vec![]
    } else {
        let src_body_str: &str = match std::str::from_utf8(src_body) {
            Ok(src) => src,
            Err(e) => {
                let err_str = format!("{}", e);
                unsafe { *err_msg_buf = err_str.as_ptr() as *const c_char };
                std::mem::forget(err_str);
                return std::ptr::null() as *const c_void;
            }
        };
        match wat::parse_str(src_body_str) {
            Ok(src) => src,
            Err(_) => {
                // Dequote the string and continue
                let src_body_dequoted = String::from(src_body_str).replace("''", "'");
                match wat::parse_str(&src_body_dequoted) {
                    Ok(src) => src,
                    Err(e) => {
                        let err_str = format!("{}", e);
                        unsafe { *err_msg_buf = err_str.as_ptr() as *const c_char };
                        std::mem::forget(err_str);
                        return std::ptr::null() as *const c_void;
                    }
                }
            }
        }
    };

    let compiled_bin = if compiled_owned.is_empty() {
        src_body
    } else {
        &compiled_owned
    };

    let module = match Module::new(unsafe { &*engine }, compiled_bin) {
        Ok(m) => m,
        Err(e) => {
            let err_str = format!("{}", e);
            unsafe { *err_msg_buf = err_str.as_ptr() as *const c_char };
            std::mem::forget(err_str);
            return std::ptr::null();
        }
    };
    let module = Box::new(module);
    let module_ptr = &*module as *const Module as *const c_void;
    std::mem::forget(module);
    module_ptr
}

#[no_mangle]
pub fn libsql_wasm_engine_new() -> *const c_void {
    let engine = match Engine::new(&Config::new()) {
        Ok(eng) => eng,
        Err(_) => return std::ptr::null() as *const c_void,
    };
    let engine = Box::new(engine);
    let engine_ptr = &*engine as *const Engine as *const c_void;
    std::mem::forget(engine);
    engine_ptr
}

#[no_mangle]
pub fn libsql_run_wasm(_context: *const c_void, _argc: i32, _argv: *mut *mut c_void) {
    println!("TODO: run wasm!");
}

#[no_mangle]
pub fn libsql_free_wasm_module(module: *mut *mut Module) {
    unsafe { Box::from_raw(*module) };
}

#[no_mangle]
pub fn libsql_wasm_free_msg_buf(err_msg_buf: *mut c_char) {
    unsafe { CString::from_raw(err_msg_buf) };
}
