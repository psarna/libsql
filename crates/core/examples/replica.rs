use libsql::Database;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    std::fs::create_dir("data.libsql").ok();
    std::fs::copy("tests/template.db", "data.libsql/data").unwrap();

    let mut db = Database::with_replicator("http://localhost:5001", "test.db")
        .await
        .unwrap();
    let conn = db.connect().unwrap();

    for _ in 0..3 {
        let sync_result = db.sync(3).await;
        println!("sync result: {:?}", sync_result);
        let response = conn.execute("SELECT * FROM sqlite_master", ()).unwrap();
        let rows = match response {
            Some(rows) => rows,
            None => {
                println!("No rows");
                continue;
            }
        };
        while let Ok(Some(row)) = rows.next() {
            println!(
                "| {:024} | {:024} | {:024} | {:024} |",
                row.get::<&str>(0).unwrap(),
                row.get::<&str>(1).unwrap(),
                row.get::<&str>(2).unwrap(),
                row.get::<&str>(3).unwrap(),
            );
        }
    }
}
