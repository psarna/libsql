#!/usr/bin/env bash

# Generates artifacts from the current build:
# - .c and .h amalgamation files
# - a precompiled binary package
#
# Assumes that ./configure and make steps were executed and succeeded

LIBSQL_WASM_UDF_SUFFIX=
if grep -s "OPT_WASM_RUNTIME = n" Makefile; then
  LIBSQL_WASM_UDF_SUFFIX=""
elif grep -s "OPT_WASM_RUNTIME = y" Makefile; then
  LIBSQL_WASM_UDF_SUFFIX="-wasm-udf"
elif grep -s "OPT_WASM_RUNTIME = d" Makefile; then
  LIBSQL_WASM_UDF_SUFFIX="-wasm-udf-dynamic"
elif grep -s "OPT_WASM_RUNTIME = wasmedge" Makefile; then
  LIBSQL_WASM_UDF_SUFFIX="-wasm-udf-wasmedge"
fi

set -x

tar czvf libsql-amalgamation-$(<LIBSQL_VERSION)${LIBSQL_WASM_UDF_SUFFIX}.tar.gz sqlite3.c sqlite3.h
tar czvf libsql-$(<LIBSQL_VERSION)${LIBSQL_WASM_UDF_SUFFIX}.tar.gz sqlite3 libsql .libs
