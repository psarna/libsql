/* SPDX-License-Identifier: MIT */
#ifdef LIBSQL_ENABLE_WASM_RUNTIME

#ifndef LIBSQL_WASM_BINDINGS_H
#define LIBSQL_WASM_BINDINGS_H

typedef struct libsql_wasm_engine_t libsql_wasm_engine_t;
typedef struct libsql_wasm_module_t libsql_wasm_module_t;

/*
** Runs a WebAssembly user-defined function.
** Additional data can be accessed via sqlite3_user_data(context)
*/
void libsql_run_wasm(sqlite3_context *context, int argc, sqlite3_value **argv);

/*
** Compiles a WebAssembly module. Can accept both .wat and binary Wasm format, depending on the implementation.
** err_msg_buf needs to be deallocated with libsql_wasm_free_msg_buf.
*/
libsql_wasm_module_t *libsql_compile_wasm_module(libsql_wasm_engine_t* engine, const char *pSrcBody, int nBody, char **err_msg_buf);

/*
** Frees an error buffer
*/
void libsql_wasm_free_msg_buf(char *err_msg_buf);

/*
** Frees a module allocated with libsql_compile_wasm_module
*/
void libsql_free_wasm_module(void *module);

/*
** Creates a new wasm engine
*/
libsql_wasm_engine_t *libsql_wasm_engine_new();

#endif //LIBSQL_WASM_BINDINGS_H
#endif //LIBSQL_ENABLE_WASM_RUNTIME
