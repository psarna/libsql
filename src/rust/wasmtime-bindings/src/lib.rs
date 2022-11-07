extern crate wasmtime;

pub use wasmtime::wasm_byte_vec_delete;
pub use wasmtime::wasmtime_error_message;
pub use wasmtime::wasm_engine_new;
pub use wasmtime::wasmtime_store_new;
pub use wasmtime::wasmtime_instance_new;
pub use wasmtime::wasmtime_instance_export_get;
pub use wasmtime::wasmtime_memory_data_size;
pub use wasmtime::wasmtime_memory_data;
pub use wasmtime::wasmtime_memory_grow;
pub use wasmtime::wasmtime_func_call;
pub use wasmtime::wasmtime_module_new;
pub use wasmtime::wasmtime_wat2wasm;
pub use wasmtime::wasmtime_module_delete;