#ifdef LIBSQL_ENABLE_WASM_RUNTIME

#include "ext/udf/wasm_bindings.h"
#include <wasm.h>
#include <wasmtime.h>

void libsql_wasm_byte_vec_delete(libsql_wasm_byte_vec_t *vec) {
    wasm_byte_vec_delete((wasm_byte_vec_t*)vec);
}

void libsql_wasm_error_message(const libsql_wasm_error_t *error, libsql_wasm_byte_vec_t *message) {
    wasmtime_error_message((wasmtime_error_t*)error, (wasm_byte_vec_t*)message);
}

libsql_wasm_engine_t *libsql_wasm_engine_new() {
    return (libsql_wasm_engine_t*)wasm_engine_new();
}

libsql_wasm_store_t *libsql_wasm_store_new(libsql_wasm_engine_t *engine, void *data, void(*finalizer)(void *)) {
    return (libsql_wasm_store_t*)wasmtime_store_new((wasm_engine_t*)engine, data, finalizer);
}

libsql_wasm_context_t *libsql_wasm_store_context(libsql_wasm_store_t *store) {
    return (libsql_wasm_context_t*)wasmtime_store_context((wasmtime_store_t*)store);
}

libsql_wasm_error_t *libsql_wasm_instance_new(
    libsql_wasm_context_t *store,
    const libsql_wasm_module_t *module,
    const libsql_wasm_extern_t* imports,
    size_t nimports,
    libsql_wasm_instance_t *instance,
    libsql_wasm_trap_t **trap
) {
    return (libsql_wasm_error_t*)wasmtime_instance_new((wasmtime_context_t*)store, (wasmtime_module_t*)module, (wasmtime_extern_t*)imports, nimports, (wasmtime_instance_t*)instance, (wasm_trap_t**)trap);
}

int libsql_wasm_instance_export_get(
    libsql_wasm_context_t *store,
    const libsql_wasm_instance_t *instance,
    const char *name,
    size_t name_len,
    libsql_wasm_extern_t *item
) {
    return wasmtime_instance_export_get((wasmtime_context_t*)store, (wasmtime_instance_t*)instance, name, name_len, (wasmtime_extern_t*)item);
}

size_t libsql_wasm_memory_data_size(
    const libsql_wasm_context_t *store,
    const libsql_wasm_memory_t *memory
) {
    return wasmtime_memory_data_size((wasmtime_context_t*)store, (wasmtime_memory_t*)memory);
}

unsigned char *libsql_wasm_memory_data(
    const libsql_wasm_context_t *store,
    const libsql_wasm_memory_t *memory
) {
    return wasmtime_memory_data((wasmtime_context_t*)store, (wasmtime_memory_t*)memory);
}

libsql_wasm_error_t *libsql_wasm_memory_grow(
    libsql_wasm_context_t *store,
    const libsql_wasm_memory_t *memory,
    uint64_t delta,
    uint64_t *prev_size
) {
    return (libsql_wasm_error_t*)wasmtime_memory_grow((wasmtime_context_t*)store, (wasmtime_memory_t*)memory, delta, prev_size);
}

libsql_wasm_error_t *libsql_wasm_func_call(
    libsql_wasm_context_t *store,
    const libsql_wasm_func_t *func,
    const libsql_wasm_val_t *args,
    size_t nargs,
    libsql_wasm_val_t *results,
    size_t nresults,
    libsql_wasm_trap_t **trap
) {
    return (libsql_wasm_error_t*)wasmtime_func_call((wasmtime_context_t*)store, (wasmtime_func_t*)func, (wasmtime_val_t*)args, nargs, (wasmtime_val_t*)results, nresults, (wasm_trap_t**)trap);
}

libsql_wasm_error_t *libsql_wasm_module_new(
    libsql_wasm_engine_t *engine,
    const uint8_t *wasm,
    size_t wasm_len,
    libsql_wasm_module_t **ret
) {
    return (libsql_wasm_error_t*)wasmtime_module_new((wasm_engine_t*)engine, wasm, wasm_len, (wasmtime_module_t**)ret);
}

libsql_wasm_error_t* libsql_wasm_wat2wasm(
    const char *wat,
    size_t wat_len,
    libsql_wasm_byte_vec_t *ret
) {
    return (libsql_wasm_error_t*)wasmtime_wat2wasm(wat, wat_len, (wasm_byte_vec_t*)ret);
}


void libsql_wasm_module_delete(libsql_wasm_module_t *m) {
    wasmtime_module_delete((wasmtime_module_t*)m);
}

#endif