#include "sqliteInt.h"
#include "wal.h"

extern int libsql_wal_methods_register(libsql_wal_methods*);

static int v_open(sqlite3_vfs *pVfs, sqlite3_file *pDbFd, const char *zWalName, int bNoShm, i64 mxWalSize, libsql_wal_methods *pMethods, Wal **ppWal) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);

  Wal *pRet;
  *ppWal = 0;
  pRet = (Wal*)sqlite3_malloc(sizeof(Wal) + pVfs->szOsFile);
  memset(pRet, 0, sizeof(*pRet));
  if( !pRet ){
    return SQLITE_NOMEM_BKPT;
  }

  pRet->pVfs = pVfs;
  pRet->pDbFd = pDbFd;
  pRet->pMethods = pMethods;

  *ppWal = pRet;
  return SQLITE_OK;
}

static int v_close(Wal *wal, sqlite3 *db, int sync_flags, int nBuf, u8 *zBuf) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  if (wal->szPage == 0) {
    return SQLITE_OK;
  }
  char *path = sqlite3_mprintf("/tmp/vwal/1");
  FILE *f = fopen(path, "r");
  sqlite3_free(path);
  if (!f) {
    return SQLITE_OK;
  }
  fprintf(stderr, "Syncing page 1\n");
  char buf[wal->szPage];
  fread(buf, 1, wal->szPage, f);
  fclose(f);
  int rc = wal->pDbFd->pMethods->xWrite(wal->pDbFd, buf, wal->szPage, 0);
  sqlite3_free(wal);
  return rc;
}

static void v_limit(Wal *wal, i64 limit) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  if (wal) {
    wal->mxWalSize = limit;
  }
}

static int v_begin_read_transaction(Wal *wal, int *) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  return SQLITE_OK;
}

static void v_end_read_transaction(Wal *wal) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);

}

static int v_find_frame(Wal *wal, Pgno pgno, u32 *frame) {
  char *path = sqlite3_mprintf("/tmp/vwal/%d", pgno);
  FILE *f = fopen(path, "r");
  sqlite3_free(path);
  *frame = 0;
  if (f) {
    fclose(f);
    *frame = pgno;
  }
  fprintf(stderr, "Looking up frame %d, found %d\n", pgno, *frame);
  return SQLITE_OK;
}

static int v_read_frame(Wal *wal, u32 frame, int nOut, u8 *pOut) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  char *path = sqlite3_mprintf("/tmp/vwal/%d", frame);
  FILE *f = fopen(path, "r");
  sqlite3_free(path);
  int n = fread(pOut, 1, nOut, f);
  fclose(f);
  fprintf(stderr, "Read %d bytes from frame %d\n", n, frame);
  return SQLITE_OK;
}

static Pgno v_dbsize(Wal *wal) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  // This abhorrent heuristics just look for pages one by one,
  // and would stop working the moment a checkpoint happens
  u32 found = 1;
  int i = 0;
  while (found) {
    i++;
    v_find_frame(wal, i, &found);
  }
  i--;
  fprintf(stderr, "DB size detected to be %d\n", i);
  return i;
}

static int v_begin_write_transaction(Wal *wal) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  return SQLITE_OK;
}

static int v_end_write_transaction(Wal *wal) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  return SQLITE_OK;
}

static int v_undo(Wal *wal, int (*xUndo)(void *, Pgno), void *pUndoCtx) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  return SQLITE_OK;
}

static void v_savepoint(Wal *wal, u32 *wal_data) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);

}

static int v_savepoint_undo(Wal *wal, u32 *wal_data) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  return SQLITE_OK;
}

static int v_frames(Wal *pWal, int szPage, PgHdr *pList, Pgno nTruncate, int isCommit, int sync_flags) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  fprintf(stderr, "nTruncate=%d\n", nTruncate);
  pWal->szPage = szPage;
  PgHdr *p;
  for (p = pList; p != NULL; p = p->pDirty) {
    fprintf(stderr, "Writing frame %d\n", p->pgno);
    char *path = sqlite3_mprintf("/tmp/vwal/%d", p->pgno);
    FILE *f = fopen(path, "w");
    sqlite3_free(path);
    int n = fwrite(p->pData, 1, szPage, f);
    fprintf(stderr, "Wrote %d bytes to frame %d\n", n, p->pgno);
    fflush(f);
    fclose(f);
  }
  return SQLITE_OK;
}

static int v_checkpoint(Wal *wal, sqlite3 *db, int eMode, int (xBusy)(void *), void *pBusyArg, int sync_flags, int nBuf, u8 *zBuf, int *pnLog, int *pnCkpt) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  return SQLITE_MISUSE;
}

static int v_callback(Wal *wal) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  return SQLITE_OK;
}

static int v_exclusive_mode(Wal *wal, int op) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  return SQLITE_OK;
}

static int v_heap_memory(Wal *wal) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  return 0;
}

// TODO: snapshot

// TODO: zipfs

static sqlite3_file *v_file(Wal *wal) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);
  return NULL;
}

//TODO: setlk timeout

static void v_db(Wal *wal, sqlite3 *db) {
  fprintf(stderr, "In %s\n", __PRETTY_FUNCTION__);

}

__attribute__((__visibility__("default")))
void libsql_register_vwal() {
  static libsql_wal_methods methods = {
    .xOpen = v_open,
    .xClose = v_close,
    .xLimit = v_limit,
    .xBeginReadTransaction = v_begin_read_transaction,
    .xEndReadTransaction = v_end_read_transaction,
    .xFindFrame = v_find_frame,
    .xReadFrame = v_read_frame,
    .xDbsize = v_dbsize,
    .xBeginWriteTransaction = v_begin_write_transaction,
    .xEndWriteTransaction = v_end_write_transaction,
    .xUndo = v_undo,
    .xSavepoint = v_savepoint,
    .xSavepointUndo = v_savepoint_undo,
    .xFrames = v_frames,
    .xCheckpoint = v_checkpoint,
    .xCallback = v_callback,
    .xExclusiveMode = v_exclusive_mode,
    .xHeapMemory = v_heap_memory,
#ifdef SQLITE_ENABLE_SNAPSHOT
    .xSnapshotGet = NULL,
    .xSnapshotOpen = NULL,
    .xSnapshotRecover = NULL,
    .xSnapshotCheck = NULL,
    .xSnapshotUnlock = NULL,
#endif
#ifdef SQLITE_ENABLE_ZIPVFS
    .xFramesize = NULL,
#endif
    .xFile = v_file,
#ifdef SQLITE_ENABLE_SETLK_TIMEOUT
    .xWriteLock = NULL,
#endif
    .xDb = v_db,
    .zName = "vwal"
  };
  libsql_wal_methods_register(&methods);
}
