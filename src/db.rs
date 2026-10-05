// Server-only SQLite connection, schema creation, and initial data loading.

use leptos::prelude::*;
use sqlx::{Connection, SqliteConnection};

/// Our default database address. "?mode=rwc" = read-write-CREATE:
/// without it, sqlx refuses to create the file if it's missing
/// (a classic "unable to open database file" surprise).
const DB_URL: &str = "sqlite:bookreader.db?mode=rwc";

/// The address can be overridden with the DATABASE_URL environment
/// variable — tests use this for throwaway databases, and deployment
/// will use it for the real server's database location.
fn db_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| DB_URL.to_string())
}

/// Open a connection to our database file (created if missing).
/// Server functions call this first. (A shared connection "pool"
/// is a later optimization — one connection per call is fine for SQLite.)
pub async fn db() -> Result<SqliteConnection, ServerFnError> {
    Ok(SqliteConnection::connect(&db_url()).await?)
}

/// Called once when the server starts (see main.rs):
/// create all tables, then fill them if the database is empty.
pub async fn init() -> Result<(), sqlx::Error> {
    let mut conn = SqliteConnection::connect(&db_url()).await?;

    // ---- Tables used TODAY (Stage 5) ----
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS books (
            id      INTEGER PRIMARY KEY AUTOINCREMENT,
            title   TEXT NOT NULL,
            author  TEXT NOT NULL DEFAULT ''
        )",
    )
    .execute(&mut conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS chapters (
            id       INTEGER PRIMARY KEY AUTOINCREMENT,
            book_id  INTEGER NOT NULL REFERENCES books(id),
            idx      INTEGER NOT NULL,   -- chapter number within the book
            title    TEXT NOT NULL,
            content  TEXT NOT NULL       -- paragraphs joined by blank lines
        )",
    )
    .execute(&mut conn)
    .await?;

    // ---- Tables for LATER stages (created now so we never
    //      need a "migration" yet — each stage just uses them) ----
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (              -- Stage 6
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            username      TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role          TEXT NOT NULL DEFAULT 'student'  -- 'student', 'parent', or 'admin'
        )",
    )
    .execute(&mut conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS sessions (           -- Stage 6
            token      TEXT PRIMARY KEY,
            user_id    INTEGER NOT NULL REFERENCES users(id),
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&mut conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS files (              -- Stage 10
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id       INTEGER NOT NULL REFERENCES users(id),
            original_name TEXT NOT NULL,
            stored_name   TEXT NOT NULL,
            mime_type     TEXT NOT NULL DEFAULT '',
            size_bytes    INTEGER NOT NULL DEFAULT 0,
            created_at    TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&mut conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS posts (              -- Stage 8
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            kind         TEXT NOT NULL DEFAULT 'blog',   -- 'blog' or 'news'
            lang         TEXT NOT NULL DEFAULT 'zh',     -- Stage 7: 'zh'/'en'
            slug         TEXT NOT NULL UNIQUE,           -- URL, e.g. /blog/my-post
            title        TEXT NOT NULL,
            content      TEXT NOT NULL,                  -- Markdown
            published_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&mut conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS subscriptions (      -- Stage 11
            user_id    INTEGER PRIMARY KEY REFERENCES users(id),
            paid_until TEXT,                             -- NULL = not paid
            provider   TEXT NOT NULL DEFAULT '',         -- 'wechat'/'alipay'
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&mut conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS classes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            notes TEXT NOT NULL DEFAULT '',
            syllabus TEXT NOT NULL DEFAULT '',
            content TEXT NOT NULL DEFAULT '',
            videos TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&mut conn)
    .await?;

    for col in ["syllabus", "content", "videos"] {
        let (has,): (i64,) = sqlx::query_as(&format!(
            "SELECT COUNT(*) FROM pragma_table_info('classes') WHERE name = '{col}'"
        ))
        .fetch_one(&mut conn)
        .await?;
        if has == 0 {
            sqlx::query(&format!(
                "ALTER TABLE classes ADD COLUMN {col} TEXT NOT NULL DEFAULT ''"
            ))
            .execute(&mut conn)
            .await?;
        }
    }

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS class_comments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            class_id INTEGER NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
            user_id INTEGER NOT NULL REFERENCES users(id),
            content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&mut conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS course_access (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id    INTEGER NOT NULL REFERENCES users(id),
            deck_id    TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(user_id, deck_id)
        )",
    )
    .execute(&mut conn)
    .await?;

    // Learner profiles belong to an account. A student normally has one;
    // a parent can add several child profiles.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS learner_profiles (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            owner_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            linked_user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
            name          TEXT NOT NULL,
            age_range     TEXT NOT NULL DEFAULT '',
            gender        TEXT NOT NULL DEFAULT '',
            created_at    TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&mut conn)
    .await?;

    let (has_linked_user_id,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pragma_table_info('learner_profiles') WHERE name = 'linked_user_id'",
    )
    .fetch_one(&mut conn)
    .await?;
    if has_linked_user_id == 0 {
        sqlx::query("ALTER TABLE learner_profiles ADD COLUMN linked_user_id INTEGER REFERENCES users(id) ON DELETE SET NULL")
            .execute(&mut conn)
            .await?;
    }
    sqlx::query(
        "CREATE UNIQUE INDEX IF NOT EXISTS learner_profiles_linked_user_unique
         ON learner_profiles(linked_user_id) WHERE linked_user_id IS NOT NULL",
    )
    .execute(&mut conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS account_link_tokens (
            token           TEXT PRIMARY KEY,
            learner_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            expires_at      TEXT NOT NULL,
            used_at         TEXT,
            created_at      TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(&mut conn)
    .await?;

    // A purchase belongs to the account; learner_courses records which
    // learner is taking that purchased course.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS course_purchases (
            id         INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            course_id  INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            UNIQUE(user_id, course_id)
        )",
    )
    .execute(&mut conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS learner_courses (
            learner_profile_id INTEGER NOT NULL REFERENCES learner_profiles(id) ON DELETE CASCADE,
            course_id           INTEGER NOT NULL,
            assigned_at         TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY(learner_profile_id, course_id)
        )",
    )
    .execute(&mut conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS course_progress (
            learner_profile_id INTEGER NOT NULL REFERENCES learner_profiles(id) ON DELETE CASCADE,
            course_id           INTEGER NOT NULL,
            deck_id             TEXT NOT NULL,
            last_slide          INTEGER NOT NULL DEFAULT 1,
            updated_at          TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY(learner_profile_id, course_id, deck_id)
        )",
    )
    .execute(&mut conn)
    .await?;

    // Give existing student accounts a learner profile, then migrate their
    // old deck grants to course-level purchases and enrollments.
    sqlx::query(
        "INSERT INTO learner_profiles (owner_user_id, name)
         SELECT u.id, u.username FROM users u
         WHERE u.role = 'student'
           AND NOT EXISTS (SELECT 1 FROM learner_profiles lp WHERE lp.owner_user_id = u.id)",
    )
    .execute(&mut conn)
    .await?;

    for course in crate::classes::ALL_COURSES {
        for deck_id in course.slide_decks {
            sqlx::query(
                "INSERT OR IGNORE INTO course_purchases (user_id, course_id)
                 SELECT user_id, ? FROM course_access WHERE deck_id = ?",
            )
            .bind(course.id)
            .bind(*deck_id)
            .execute(&mut conn)
            .await?;
        }
        sqlx::query(
            "INSERT OR IGNORE INTO learner_courses (learner_profile_id, course_id)
             SELECT lp.id, cp.course_id
             FROM learner_profiles lp
             JOIN course_purchases cp ON cp.user_id = lp.owner_user_id
             WHERE cp.course_id = ?",
        )
        .bind(course.id)
        .execute(&mut conn)
        .await?;
    }

    // First run only: fill the database from books/book.txt.
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books")
        .fetch_one(&mut conn)
        .await?;
    if count == 0 {
        seed(&mut conn).await?;
    }

    // Stage 8: also make sure there are a couple of example posts,
    // so the blog/news pages aren't empty on first visit.
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts")
        .fetch_one(&mut conn)
        .await?;
    if count == 0 {
        seed_posts(&mut conn).await?;
    }

    Ok(())
}

/// Two example posts (in Chinese — also a nice end-to-end UTF-8 test).
/// Replace or delete them from the admin page once you're live.
async fn seed_posts(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO posts (kind, lang, slug, title, content) VALUES (?, ?, ?, ?, ?)",
    )
    .bind("blog")
    .bind("zh")
    .bind("welcome")
    .bind("欢迎来到星阅读")
    .bind("## 欢迎\n\n这里是星阅读博客，我们会分享：\n\n- 孩子们的阅读故事\n- 阅读技巧\n- 家长问答\n\n**欢迎关注！**")
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "INSERT INTO posts (kind, lang, slug, title, content) VALUES (?, ?, ?, ?, ?)",
    )
    .bind("news")
    .bind("zh")
    .bind("autumn-courses")
    .bind("秋季课程开始报名")
    .bind("## 秋季课程\n\n新学期阅读课程现已开放报名，欢迎五年级同学加入！\n\n名额有限，先到先得。")
    .execute(&mut *conn)
    .await?;

    Ok(())
}

/// Read books/book.txt with the Stage 3 parser and INSERT it as
/// one books row + one chapters row per chapter.
async fn seed(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
    let (title, chapters) = load_book();

    let book_id = sqlx::query("INSERT INTO books (title, author) VALUES (?, ?)")
        .bind(title)
        .bind("Emma Example")
        .execute(&mut *conn)
        .await?
        .last_insert_rowid(); // the id SQLite just gave our new book

    for (i, (title, paragraphs)) in chapters.iter().enumerate() {
        sqlx::query(
            "INSERT INTO chapters (book_id, idx, title, content) VALUES (?, ?, ?, ?)",
        )
        .bind(book_id)
        .bind((i + 1) as i64) // chapters are 1-based
        .bind(title)
        .bind(paragraphs.join("\n\n")) // store paragraphs joined by blank lines
        .execute(&mut *conn) // &mut * = re-borrow, so the loop can use conn again
        .await?;
    }

    Ok(())
}

// ---- The Stage 3 parser, moved here (it's storage's job now) ----

fn load_book() -> (String, Vec<(String, Vec<String>)>) {
    let text = std::fs::read_to_string("books/book.txt").unwrap_or_default();
    parse_book(&text)
}

/// Our import convention: a line like "Chapter 12" ALL BY ITSELF
/// starts a new chapter. A paragraph that merely STARTS with the
/// word "Chapter" (e.g. "Chapter quizzes are fun.") must NOT count —
/// a bug our unit tests caught before users did.
fn is_chapter_heading(line: &str) -> bool {
    line.strip_prefix("Chapter ")
        .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
}

fn parse_book(text: &str) -> (String, Vec<(String, Vec<String>)>) {
    let mut chapters = Vec::new();
    let mut book_title = None;
    let mut chapter_title = None;
    let mut paragraphs = Vec::new();
    let mut current_lines = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            if !current_lines.is_empty() {
                paragraphs.push(current_lines.join(" "));
                current_lines.clear();
            }
        } else if is_chapter_heading(line) {
            if !current_lines.is_empty() {
                paragraphs.push(current_lines.join(" "));
                current_lines.clear();
            }
            if let Some(title) = chapter_title.take() {
                chapters.push((title, std::mem::take(&mut paragraphs)));
            } else if book_title.is_none() {
                book_title = paragraphs.first().cloned();
                paragraphs.clear();
            }
            chapter_title = Some(line.to_string());
        } else {
            current_lines.push(line.to_string());
        }
    }

    if !current_lines.is_empty() {
        paragraphs.push(current_lines.join(" "));
    }
    if let Some(title) = chapter_title {
        chapters.push((title, paragraphs));
    }

    (book_title.unwrap_or_else(|| "Untitled".to_string()), chapters)
}

// ----------------------------------------------------------
//  UNIT TESTS — run with: cargo test --features ssr
//  These check the parser against tricky inputs so future
//  changes can't silently break book importing (regression).
// ----------------------------------------------------------

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn parses_title_chapters_and_paragraphs() {
        let text = "My Book Title\n\nChapter 1\n\nFirst paragraph.\n\nSecond paragraph.\n\nChapter 2\n\nChapter two text.";
        let (title, chapters) = parse_book(text);

        assert_eq!(title, "My Book Title");
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].0, "Chapter 1");
        assert_eq!(chapters[0].1, vec!["First paragraph.", "Second paragraph."]);
        assert_eq!(chapters[1].0, "Chapter 2");
        assert_eq!(chapters[1].1, vec!["Chapter two text."]);
    }

    #[test]
    fn joins_wrapped_lines_into_one_paragraph() {
        // a paragraph spanning several lines (no blank line between)
        let (_, chapters) = parse_book("Chapter 1\n\nline one\nline two");
        assert_eq!(chapters[0].1, vec!["line one line two"]);
    }

    #[test]
    fn handles_missing_title_preamble() {
        let (title, chapters) = parse_book("Chapter 1\n\nSome text.");
        assert_eq!(title, "Untitled"); // fallback
        assert_eq!(chapters.len(), 1);
    }

    #[test]
    fn paragraph_starting_with_chapter_is_not_a_heading() {
        let (_, chapters) = parse_book("Chapter 1\n\nChapter quizzes are fun.");
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].1, vec!["Chapter quizzes are fun."]);
    }

    #[test]
    fn empty_file_gives_zero_chapters() {
        let (_, chapters) = parse_book("");
        assert_eq!(chapters.len(), 0);
    }
}
