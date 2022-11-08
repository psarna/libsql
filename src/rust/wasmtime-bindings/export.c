typedef struct FuncDef FuncDef;
typedef struct sqlite3 sqlite3;
typedef struct sqlite3_value sqlite3_value;
typedef struct sqlite3_context sqlite3_context;

__attribute__ ((visibility ("default"))) FuncDef *try_instantiate_wasm_function(sqlite3 *db, const char *pName, int nName, const char *pSrcBody, int nBody, int nArg, char **err_msg_buf);

/*
** Runs a WebAssembly user-defined function.
** Additional data can be accessed via sqlite3_user_data(context)
*/
__attribute__ ((visibility ("default"))) void run_wasm(sqlite3_context *context, int argc, sqlite3_value **argv);

