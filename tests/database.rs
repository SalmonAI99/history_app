// Integration tests: run the REAL database layer against real
// (temporary) SQLite files. Run with:
//
//     cargo test --features ssr
//
// ("--features ssr" because the db module only exists in the server
// build — see lib.rs.)

#![cfg(feature = "ssr")]

use bookreader::db;

/// A unique temporary database path per test-run.
fn temp_db_url(name: &str) -> (String, std::path::PathBuf) {
    let path = std::env::temp_dir().join(format!("bookreader_test_{name}_{}.db", std::process::id()));
    (format!("sqlite:{}?mode=rwc", path.display()), path)
}

// One test function, several sections — DATABASE_URL is a global
// environment variable, so sequential checks inside one function
// can't interfere with each other (parallel tests could!).
#[tokio::test]
async fn database_layer_works() {
    // ---------- Section 1: init creates all tables + seeds the book ----------
    let (url, path) = temp_db_url("init");
    std::env::set_var("DATABASE_URL", &url);

    db::init().await.expect("init should succeed");
    let mut conn = db::db().await.unwrap();

    let tables: Vec<(String,)> =
        sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .fetch_all(&mut conn)
            .await
            .unwrap();
    let names: Vec<&str> = tables.iter().map(|t| t.0.as_str()).collect();
    for needed in [
        "books", "chapters", "users", "sessions",
        "progress", "documents", "files", "posts", "subscriptions",
    ] {
        assert!(names.contains(&needed), "missing table: {needed}");
    }

    let (books,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert!(books >= 1, "seed should create a book");

    let (chapters,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chapters WHERE book_id = 1")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    assert!(chapters >= 1, "seed should create chapters");

    drop(conn);
    let _ = std::fs::remove_file(&path);

    // ---------- Section 2: usernames must be unique ----------
    // (this is the constraint the signup server function relies on
    //  to answer "that username is taken")
    let (url, path) = temp_db_url("unique");
    std::env::set_var("DATABASE_URL", &url);

    db::init().await.unwrap();
    let mut conn = db::db().await.unwrap();

    sqlx::query("INSERT INTO users (username, password_hash, role) VALUES ('tester', 'hash1', 'student')")
        .execute(&mut conn)
        .await
        .unwrap();

    let duplicate =
        sqlx::query("INSERT INTO users (username, password_hash, role) VALUES ('tester', 'hash2', 'student')")
            .execute(&mut conn)
            .await;
    assert!(duplicate.is_err(), "duplicate username should be rejected");

    drop(conn);
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn password_hashing_is_one_way_and_verifies() {
    let hash = bcrypt::hash("secret123", bcrypt::DEFAULT_COST).unwrap();

    // correct password verifies, wrong password doesn't:
    assert!(bcrypt::verify("secret123", &hash).unwrap());
    assert!(!bcrypt::verify("wrongpass", &hash).unwrap());

    // and the password itself must NOT appear anywhere in the hash:
    assert!(!hash.contains("secret123"));
}
