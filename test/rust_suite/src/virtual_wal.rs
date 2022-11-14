#[cfg(test)]
mod tests {
    use rusqlite::Connection;
    use std::ffi::c_void;

    #[repr(C)]
    struct Wal {
        vfs: *const c_void,
        db_fd: *const c_void,
        wal_fd: *const c_void,
        callback_value: u32,
        max_wal_size: i64,
        wi_data: i32,
        size_first_block: i32,
        ap_wi_data: *const *mut u32,
        page_size: u32,
        read_lock: i16,
        sync_flags: u8,
        exclusive_mode: u8,
        write_lock: u8,
        checkpoint_lock: u8,
        read_only: u8,
        truncate_on_commit: u8,
        sync_header: u8,
        pad_to_section_boundary: u8,
        b_shm_unreliable: u8,
        hdr: WalIndexHdr,
        min_frame: u32,
        recalculate_checksums: u32,
        wal_name: *const u8,
        n_checkpoints: u32,
        // debug: log_error
        // snapshot: p_snapshot
        // setlk: *db
        wal_methods: *const libsql_wal_methods,
    }

    #[repr(C)]
    struct WalIndexHdr {
        version: u32,
        unused: u32,
        change: u32,
        is_init: u8,
        big_endian_checksum: u8,
        page_size: u16,
        last_valid_frame: u32,
        n_pages: u32,
        frame_checksum: [u32; 2],
        salt: [u32; 2],
        checksum: [u32; 2],
    }

    #[repr(C)]
    struct libsql_wal_methods {
        open: extern "C" fn(
            vfs: *const c_void,
            file: *const c_void,
            wal_name: *const u8,
            no_shm_mode: i32,
            max_size: i64,
            methods: *const libsql_wal_methods,
            wal: *mut *const Wal,
        ) -> i32,
        close: extern "C" fn(
            wal: *mut Wal,
            db: *mut c_void,
            sync_flags: i32,
            n_buf: i32,
            z_buf: *mut u8,
        ) -> i32,
        limit: extern "C" fn(wal: *mut Wal, limit: i64),
        begin_read: extern "C" fn(wal: *mut Wal, changed: *mut i32) -> i32,
        end_read: extern "C" fn(wal: *mut Wal) -> i32,
        find_frame: extern "C" fn(wal: *mut Wal, pgno: i32, frame: *mut i32) -> i32,
        read_frame: extern "C" fn(wal: *mut Wal, frame: u32, n_out: i32, p_out: *mut u8) -> i32,
        db_size: extern "C" fn(wal: *mut Wal) -> i32,
        begin_write: extern "C" fn(wal: *mut Wal) -> i32,
        end_write: extern "C" fn(wal: *mut Wal) -> i32,
        undo: extern "C" fn(
            wal: *const extern "C" fn(*mut c_void, i32) -> i32,
            ctx: *mut c_void,
        ) -> i32,
        savepoint: extern "C" fn(wal: *mut Wal, wal_data: *mut u32),
        savepoint_undo: extern "C" fn(wal: *mut Wal, wal_data: *mut u32) -> i32,
        frames: extern "C" fn(
            wal: *mut Wal,
            page_size: i32,
            page_headers: *const PgHdr,
            size_after: i32,
            is_commit: i32,
            sync_flags: i32,
        ) -> i32,
        checkpoint: extern "C" fn(
            wal: *mut Wal,
            db: *mut c_void,
            emode: i32,
            busy_handler: extern "C" fn(busy_param: *mut c_void) -> i32,
            sync_flags: i32,
            n_buf: i32,
            z_buf: *mut u8,
            frames_in_wal: *mut i32,
            backfilled_frames: *mut i32,
        ) -> i32,
        callback: extern "C" fn(wal: *mut Wal) -> i32,
        exclusive_mode: extern "C" fn(wal: *mut Wal) -> i32,
        heap_memory: extern "C" fn(wal: *mut Wal) -> i32,
        // snapshot: get, open, recover, check, unlock
        // enable_zipvfs: framesize
        file: extern "C" fn(wal: *mut Wal) -> *const c_void,
        db: extern "C" fn(wal: *mut Wal, db: *const c_void),
        pathname_len: extern "C" fn(orig_len: i32) -> i32,
        get_pathname: extern "C" fn(buf: *mut u8, orig: *const u8, orig_len: i32),
        b_uses_shm: i32,
        name: *const u8,
        p_next: *const c_void,
    }

    #[repr(C)]
    struct PgHdr {
        page: *const c_void,
        data: *const c_void,
        extra: *const c_void,
        pcache: *const c_void,
        dirty: *const PgHdr,
        pager: *const c_void,
        pgno: i32,
        flags: u16,
    }

    extern "C" {
        fn libsql_open(
            filename: *const u8,
            ppdb: *mut *mut rusqlite::ffi::sqlite3,
            flags: i32,
            vfs: *const u8,
            wal: *const u8,
        ) -> i32;
        fn libsql_wal_methods_register(wal_methods: *const libsql_wal_methods) -> i32;
        fn sqlite3_initialize();
    }

    extern "C" fn open(
        vfs: *const c_void,
        _file: *const c_void,
        wal_name: *const u8,
        _no_shm_mode: i32,
        max_size: i64,
        methods: *const libsql_wal_methods,
        wal: *mut *const Wal,
    ) -> i32 {
        let new_wal = Box::new(Wal {
            vfs: vfs,
            db_fd: std::ptr::null(),
            wal_fd: std::ptr::null(),
            callback_value: 0,
            max_wal_size: max_size,
            wi_data: 0,
            size_first_block: 0,
            ap_wi_data: std::ptr::null(),
            page_size: 4096,
            read_lock: 0,
            sync_flags: 0,
            exclusive_mode: 1,
            write_lock: 0,
            checkpoint_lock: 0,
            read_only: 0,
            truncate_on_commit: 0,
            sync_header: 0,
            pad_to_section_boundary: 0,
            b_shm_unreliable: 1,
            hdr: WalIndexHdr {
                version: 1,
                unused: 0,
                change: 0,
                is_init: 0,
                big_endian_checksum: 0,
                page_size: 4096,
                last_valid_frame: 1,
                n_pages: 1,
                frame_checksum: [0, 0],
                salt: [0, 0],
                checksum: [0, 0],
            },
            min_frame: 0,
            recalculate_checksums: 0,
            wal_name: wal_name,
            n_checkpoints: 0,
            wal_methods: methods,
        });
        unsafe { *wal = &*new_wal }
        Box::leak(new_wal);
        0
    }
    extern "C" fn close(
        _wal: *mut Wal,
        _db: *mut c_void,
        _sync_flags: i32,
        _n_buf: i32,
        _z_buf: *mut u8,
    ) -> i32 {
        println!("Closing WAL");
        0
    }
    extern "C" fn limit(_wal: *mut Wal, limit: i64) {
        println!("Limit: {}", limit);
    }
    extern "C" fn begin_read(_wal: *mut Wal, changed: *mut i32) -> i32 {
        println!("Read started");
        unsafe { *changed = 1 }
        0
    }
    extern "C" fn end_read(_wal: *mut Wal) -> i32 {
        println!("Read ended");
        0
    }
    extern "C" fn find_frame(_wal: *mut Wal, pgno: i32, frame: *mut i32) -> i32 {
        println!("Looking for page {}", pgno);
        unsafe { *frame = pgno };
        0
    }
    extern "C" fn read_frame(_wal: *mut Wal, frame: u32, _n_out: i32, _p_out: *mut u8) -> i32 {
        println!("Reading frame {}", frame);
        0
    }
    extern "C" fn db_size(_wal: *mut Wal) -> i32 {
        21
    }
    extern "C" fn begin_write(_wal: *mut Wal) -> i32 {
        println!("Write started");
        0
    }
    extern "C" fn end_write(_wal: *mut Wal) -> i32 {
        println!("Write ended");
        0
    }
    extern "C" fn undo(
        _wal: *const extern "C" fn(*mut c_void, i32) -> i32,
        _ctx: *mut c_void,
    ) -> i32 {
        21
    }
    extern "C" fn savepoint(_wal: *mut Wal, _wal_data: *mut u32) {}
    extern "C" fn savepoint_undo(_wal: *mut Wal, _wal_data: *mut u32) -> i32 {
        21
    }
    extern "C" fn frames(
        _wal: *mut Wal,
        _page_size: i32,
        page_headers: *const PgHdr,
        _size_after: i32,
        _is_commit: i32,
        _sync_flags: i32,
    ) -> i32 {
        println!("Writing frames...");
        let mut current_ptr = page_headers;
        loop {
            let current: &PgHdr = unsafe { &*current_ptr };
            println!("\tpage {}", current.pgno);
            if current.dirty == std::ptr::null() {
                break
            }
            current_ptr = current.dirty
        }
        0
    }
    extern "C" fn checkpoint(
        _wal: *mut Wal,
        _db: *mut c_void,
        _emode: i32,
        _busy_handler: extern "C" fn(busy_param: *mut c_void) -> i32,
        _sync_flags: i32,
        _n_buf: i32,
        _z_buf: *mut u8,
        _frames_in_wal: *mut i32,
        _backfilled_frames: *mut i32,
    ) -> i32 {
        println!("Checkpointed");
        0
    }
    extern "C" fn callback(_wal: *mut Wal) -> i32 {
        21
    }
    extern "C" fn exclusive_mode(_wal: *mut Wal) -> i32 {
        1
    }
    extern "C" fn heap_memory(_wal: *mut Wal) -> i32 {
        21
    }
    extern "C" fn file(_wal: *mut Wal) -> *const c_void {
        panic!("Should never be called")
    }
    extern "C" fn db(_wal: *mut Wal, _db: *const c_void) {}
    extern "C" fn pathname_len(_orig_len: i32) -> i32 {
        println!("Returning length 0");
        0
    }
    extern "C" fn get_pathname(_buf: *mut u8, _orig: *const u8, _orig_len: i32) {
        panic!("Should never be called")
    }

    #[test]
    fn test_vwal_register() {
        let conn = unsafe {
            let mut pdb: *mut rusqlite::ffi::sqlite3 = std::ptr::null_mut();
            let ppdb: *mut *mut rusqlite::ffi::sqlite3 = &mut pdb;
            let vwal = Box::new(libsql_wal_methods {
                open: open,
                close: close,
                limit: limit,
                begin_read: begin_read,
                end_read: end_read,
                find_frame: find_frame,
                read_frame: read_frame,
                db_size: db_size,
                begin_write: begin_write,
                end_write: end_write,
                undo: undo,
                savepoint: savepoint,
                savepoint_undo: savepoint_undo,
                frames: frames,
                checkpoint: checkpoint,
                callback: callback,
                exclusive_mode: exclusive_mode,
                heap_memory: heap_memory,
                file: file,
                db: db,
                pathname_len: pathname_len,
                get_pathname: get_pathname,
                b_uses_shm: 0,
                name: "vwal\0".as_ptr(),
                p_next: std::ptr::null(),
            });

            sqlite3_initialize();
            let register_err = libsql_wal_methods_register(&*vwal as *const libsql_wal_methods);
            assert_eq!(register_err, 0);
            let open_err = libsql_open(
                "/tmp/heyyyy\0".as_ptr(),
                ppdb,
                6,
                std::ptr::null(),
                "vwal\0".as_ptr(),
            );
            assert_eq!(open_err, 0);
            Box::leak(vwal);
            Connection::from_handle(pdb).unwrap()
        };

        conn.pragma_update(None, "journal_mode", "wal").unwrap();
        conn.execute("CREATE TABLE t(id)", ()).unwrap();
        conn.execute("INSERT INTO t(id) VALUES (42)", ()).unwrap();
        conn.execute("INSERT INTO t(id) VALUES (zeroblob(8193))", ())
            .unwrap();
        conn.execute("INSERT INTO t(id) VALUES (7.0)", ()).unwrap();

        let seven: f64 = conn
            .query_row("SELECT id FROM t WHERE id = 7.0", [], |r| r.get(0))
            .unwrap();
        println!("Seven: {}", seven);
    }
}
