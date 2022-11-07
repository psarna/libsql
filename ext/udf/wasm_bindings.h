/* SPDX-License-Identifier: MIT */
#ifdef LIBSQL_ENABLE_WASM_RUNTIME

#ifndef LIBSQL_WASM_BINDINGS_H
#define LIBSQL_WASM_BINDINGS_H

typedef struct libsql_wasm_engine_t libsql_wasm_engine_t;

/*
** Tries to instantiate a WebAssembly user-defined function.
** Additional data can be passed in FuncDef::pUserData.
** If err_msg_buf is not null, it will be allocated on error
** and contain the error message, which must be freed later
** with sqlite3DbFree.
*/
typedef struct FuncDef FuncDef;
FuncDef *try_instantiate_wasm_function(sqlite3 *db, const char *pName, int nName, const char *pSrcBody, int nBody, int nArg, char **err_msg_buf);

/*
** Runs a WebAssembly user-defined function.
** Additional data can be accessed via sqlite3_user_data(context)
*/
void run_wasm(sqlite3_context *context, int argc, sqlite3_value **argv);

int deregister_wasm_function(sqlite3 *db, const char *zName);

#endif //LIBSQL_WASM_BINDINGS_H
#endif //LIBSQL_ENABLE_WASM_RUNTIME
