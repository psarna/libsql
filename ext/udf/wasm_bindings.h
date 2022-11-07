/* SPDX-License-Identifier: MIT */

#ifndef LIBSQL_WASM_BINDINGS_H
#define LIBSQL_WASM_BINDINGS_H

typedef uint8_t libsql_wasm_valkind_t;

#define LIBSQL_WASM_I32       0
#define LIBSQL_WASM_I64       1
#define LIBSQL_WASM_F32       2
#define LIBSQL_WASM_F64       3
#define LIBSQL_WASM_V128      4
#define LIBSQL_WASM_FUNCREF   5
#define LIBSQL_WASM_EXTERNREF 6

#define LIBSQL_WASM_EXTERN_FUNC   0
#define LIBSQL_WASM_EXTERN_GLOBAL 1
#define LIBSQL_WASM_EXTERN_TABLE  2
#define LIBSQL_WASM_EXTERN_MEMORY 3

typedef struct libsql_wasm_externref libsql_wasm_externref_t;

typedef struct libsql_wasm_func {
  uint64_t store_id;
  size_t index;
} libsql_wasm_func_t;
 
typedef struct libsql_wasm_table {
  uint64_t store_id;
  size_t index;
} libsql_wasm_table_t;
 
typedef struct libsql_wasm_memory {
  uint64_t store_id;
  size_t index;
} libsql_wasm_memory_t;
 
typedef struct libsql_wasm_global {
  uint64_t store_id;
  size_t index;
} libsql_wasm_global_t;

typedef union libsql_wasm_extern_union {
    libsql_wasm_func_t func;
    libsql_wasm_global_t global;
    libsql_wasm_table_t table;
    libsql_wasm_memory_t memory;
} libsql_wasm_extern_union_t;

typedef uint8_t libsql_wasm_extern_kind_t;

typedef struct libsql_wasm_extern {
    libsql_wasm_extern_kind_t kind;
    libsql_wasm_extern_union_t of;
} libsql_wasm_extern_t;
 
typedef uint8_t libsql_wasm_extern_kind_t;

typedef uint8_t libsql_wasm_v128[16];
 
typedef union libsql_wasm_valunion {
  int32_t i32;
  int64_t i64;
  float f32;
  double f64;
  libsql_wasm_func_t funcref;
  libsql_wasm_externref_t *externref;
  libsql_wasm_v128 v128;
} libsql_wasm_valunion_t;
 
typedef union libsql_wasm_val_raw {
  int32_t i32;
  int64_t i64;
  float f32;
  double f64;
  libsql_wasm_v128 v128;
  size_t funcref;
  size_t externref;
} libsql_wasm_val_raw_t;
 
typedef struct libsql_wasm_val {
  libsql_wasm_valkind_t kind;
  libsql_wasm_valunion_t of;
} libsql_wasm_val_t;

typedef struct libsql_wasm_error_t libsql_wasm_error_t;

typedef struct libsql_wasm_byte_vec_t {
  size_t size;
  uint8_t *data;
} libsql_wasm_byte_vec_t;

void libsql_wasm_byte_vec_delete(libsql_wasm_byte_vec_t *vec);

void libsql_wasm_error_message(const libsql_wasm_error_t *error, libsql_wasm_byte_vec_t *message);

typedef struct libsql_wasm_module_t libsql_wasm_module_t;
typedef struct libsql_wasm_engine_t libsql_wasm_engine_t;
typedef struct libsql_wasm_store_t libsql_wasm_store_t;
typedef struct libsql_wasm_context_t libsql_wasm_context_t;
typedef struct libsql_wasm_trap_t libsql_wasm_trap_t;

typedef struct libsql_wasm_instance {
  uint64_t store_id;
  size_t index;
} libsql_wasm_instance_t;

libsql_wasm_engine_t *libsql_wasm_engine_new();

libsql_wasm_store_t *libsql_wasm_store_new(libsql_wasm_engine_t *engine, void *data, void(*finalizer)(void *));
libsql_wasm_context_t *libsql_wasm_store_context(libsql_wasm_store_t *store);
libsql_wasm_error_t *libsql_wasm_instance_new(
    libsql_wasm_context_t *store,
    const libsql_wasm_module_t *module,
    const libsql_wasm_extern_t* imports,
    size_t nimports,
    libsql_wasm_instance_t *instance,
    libsql_wasm_trap_t **trap
);

int libsql_wasm_instance_export_get(
    libsql_wasm_context_t *store,
    const libsql_wasm_instance_t *instance,
    const char *name,
    size_t name_len,
    libsql_wasm_extern_t *item
);

size_t libsql_wasm_memory_data_size(
    const libsql_wasm_context_t *store,
    const libsql_wasm_memory_t *memory
);

unsigned char *libsql_wasm_memory_data(
    const libsql_wasm_context_t *store,
    const libsql_wasm_memory_t *memory
);

libsql_wasm_error_t *libsql_wasm_memory_grow(
    libsql_wasm_context_t *store,
    const libsql_wasm_memory_t *memory,
    uint64_t delta,
    uint64_t *prev_size
);

libsql_wasm_error_t *libsql_wasm_func_call(
    libsql_wasm_context_t *store,
    const libsql_wasm_func_t *func,
    const libsql_wasm_val_t *args,
    size_t nargs,
    libsql_wasm_val_t *results,
    size_t nresults,
    libsql_wasm_trap_t **trap
);

libsql_wasm_error_t *libsql_wasm_module_new(
    libsql_wasm_engine_t *engine,
    const uint8_t *wasm,
    size_t wasm_len,
    libsql_wasm_module_t **ret
);

libsql_wasm_error_t* libsql_wasm_wat2wasm(
    const char *wat,
    size_t wat_len,
    libsql_wasm_byte_vec_t *ret
);


void libsql_wasm_module_delete(libsql_wasm_module_t *m);

#endif
