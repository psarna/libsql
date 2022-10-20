/*
** 2010 February 1
**
** The author disclaims copyright to this source code.  In place of
** a legal notice, here is a blessing:
**
**    May you do good and not evil.
**    May you find forgiveness for yourself and forgive others.
**    May you share freely, never taking more than you give.
**
*************************************************************************
** This header file defines the interface to the write-ahead logging 
** system. Refer to the comments below and the header comment attached to 
** the implementation of each function in log.c for further details.
*/

#ifndef SQLITE_WAL_H
#define SQLITE_WAL_H

#include "sqliteInt.h"

/* Macros for extracting appropriate sync flags for either transaction
** commits (WAL_SYNC_FLAGS(X)) or for checkpoint ops (CKPT_SYNC_FLAGS(X)):
*/
#define WAL_SYNC_FLAGS(X)   ((X)&0x03)
#define CKPT_SYNC_FLAGS(X)  (((X)>>2)&0x03)

#define WAL_SAVEPOINT_NDATA 4

/* Connection to a write-ahead log (WAL) file. 
** There is one object of this type for each pager. 
*/
typedef struct Wal Wal;

typedef struct libsql_wal_methods {
  /* Open and close a connection to a write-ahead log. */
  int (*xOpen)(sqlite3_vfs*, sqlite3_file* , const char*, int no_shm_mode, i64 max_size, struct libsql_wal_methods*, Wal**);
  int (*xClose)(Wal*, sqlite3* db, int sync_flags, int nBuf, u8 *zBuf);

  /* Set the limiting size of a WAL file. */
  void (*xLimit)(Wal*, i64 limit);

  /* Used by readers to open (lock) and close (unlock) a snapshot.  A 
  ** snapshot is like a read-transaction.  It is the state of the database
  ** at an instant in time.  sqlite3WalOpenSnapshot gets a read lock and
  ** preserves the current state even if the other threads or processes
  ** write to or checkpoint the WAL.  sqlite3WalCloseSnapshot() closes the
  ** transaction and releases the lock.
  */
  int (*xBeginReadTransaction)(Wal *, int *);
  void (*xEndReadTransaction)(Wal *);

  /* Read a page from the write-ahead log, if it is present. */
  int (*xFindFrame)(Wal *, Pgno, u32 *);
  int (*xReadFrame)(Wal *, u32, int, u8 *);

  /* If the WAL is not empty, return the size of the database. */
  Pgno (*xDbsize)(Wal *pWal);

  /* Obtain or release the WRITER lock. */
  int (*xBeginWriteTransaction)(Wal *pWal);
  int (*xEndWriteTransaction)(Wal *pWal);

  /* Undo any frames written (but not committed) to the log */
  int (*xUndo)(Wal *pWal, int (*xUndo)(void *, Pgno), void *pUndoCtx);

  /* Return an integer that records the current (uncommitted) write
  ** position in the WAL */
  void (*xSavepoint)(Wal *pWal, u32 *aWalData);

  /* Move the write position of the WAL back to iFrame.  Called in
  ** response to a ROLLBACK TO command. */
  int (*xSavepointUndo)(Wal *pWal, u32 *aWalData);

  /* Write a frame or frames to the log. */
  int (*xFrames)(Wal *pWal, int, PgHdr *, Pgno, int, int);

  /* Copy pages from the log to the database file */ 
  int (*xCheckpoint)(
    Wal *pWal,                      /* Write-ahead log connection */
    sqlite3 *db,                    /* Check this handle's interrupt flag */
    int eMode,                      /* One of PASSIVE, FULL and RESTART */
    int (*xBusy)(void*),            /* Function to call when busy */
    void *pBusyArg,                 /* Context argument for xBusyHandler */
    int sync_flags,                 /* Flags to sync db file with (or 0) */
    int nBuf,                       /* Size of buffer nBuf */
    u8 *zBuf,                       /* Temporary buffer to use */
    int *pnLog,                     /* OUT: Number of frames in WAL */
    int *pnCkpt                     /* OUT: Number of backfilled frames in WAL */
  );

  /* Return the value to pass to a sqlite3_wal_hook callback, the
  ** number of frames in the WAL at the point of the last commit since
  ** sqlite3WalCallback() was called.  If no commits have occurred since
  ** the last call, then return 0.
  */
  int (*xCallback)(Wal *pWal);

  /* Tell the wal layer that an EXCLUSIVE lock has been obtained (or released)
  ** by the pager layer on the database file.
  */
  int (*xExclusiveMode)(Wal *pWal, int op);

  /* Return true if the argument is non-NULL and the WAL module is using
  ** heap-memory for the wal-index. Otherwise, if the argument is NULL or the
  ** WAL module is using shared-memory, return false. 
  */
  int (*xHeapMemory)(Wal *pWal);

#ifdef SQLITE_ENABLE_SNAPSHOT
  int (*xSnapshotGet)(Wal *pWal, sqlite3_snapshot **ppSnapshot);
  void (*xSnapshotOpen)(Wal *pWal, sqlite3_snapshot *pSnapshot);
  int (*xSnapshotRecover)(Wal *pWal);
  int (*xSnapshotCheck)(Wal *pWal, sqlite3_snapshot *pSnapshot);
  void (*xSnapshotUnlock)(Wal *pWal);
#endif

#ifdef SQLITE_ENABLE_ZIPVFS
  /* If the WAL file is not empty, return the number of bytes of content
  ** stored in each frame (i.e. the db page-size when the WAL was created).
  */
  int (*xFramesize)(Wal *pWal);
#endif

  /* Return the sqlite3_file object for the WAL file */
  sqlite3_file *(*xFile)(Wal *pWal);

#ifdef SQLITE_ENABLE_SETLK_TIMEOUT
  int (*xWriteLock)(Wal *pWal, int bLock);
#endif

  void (*xDb)(Wal *pWal, sqlite3 *db);

  const char *zName;
} libsql_wal_methods;

libsql_wal_methods* libsql_wal_methods_find(const char *zName);

#endif /* SQLITE_WAL_H */
