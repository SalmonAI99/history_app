// Server-only SQLite connection, schema creation, and initial data loading.

use leptos::prelude::*;
use sqlx::{Connection, SqliteConnection};

use sqlx::migrate::Migrator;

static MIGRATOR: Migrator = sqlx::migrate!(); // defaults to "./migrations"

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

    MIGRATOR.run(&mut conn).await?;


    // for course in crate::classes::ALL_COURSES {
    //     for deck_id in course.slide_decks {
    //         sqlx::query(
    //             "INSERT OR IGNORE INTO course_purchases (user_id, course_id)
    //              SELECT user_id, ? FROM course_access WHERE deck_id = ?",
    //         )
    //         .bind(course.id)
    //         .bind(*deck_id)
    //         .execute(&mut conn)
    //         .await?;
    //     }
    //     sqlx::query(
    //         "INSERT OR IGNORE INTO learner_courses (learner_profile_id, course_id)
    //          SELECT lp.id, cp.course_id
    //          FROM learner_profiles lp
    //          JOIN course_purchases cp ON cp.user_id = lp.owner_user_id
    //          WHERE cp.course_id = ?",
    //     )
    //     .bind(course.id)
    //     .execute(&mut conn)
    //     .await?;
    // }

    // First run only: fill the database from books/book.txt.
    // let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM books")
    //     .fetch_one(&mut conn)
    //     .await?;
    // if count == 0 {
    //     seed(&mut conn).await?;
    // }
    //
    // // Stage 8: also make sure there are a couple of example posts,
    // // so the blog/news pages aren't empty on first visit.
    // let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM posts")
    //     .fetch_one(&mut conn)
    //     .await?;
    // if count == 0 {
    //     seed_posts(&mut conn).await?;
    // }

    Ok(())
}

/// Two example posts (in Chinese — also a nice end-to-end UTF-8 test).
/// Replace or delete them from the admin page once you're live.
// async fn seed_posts(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
//     sqlx::query(
//         "INSERT INTO posts (kind, lang, slug, title, content) VALUES (?, ?, ?, ?, ?)",
//     )
//     .bind("blog")
//     .bind("zh")
//     .bind("welcome")
//     .bind("欢迎来到星阅读")
//     .bind("## 欢迎\n\n这里是星阅读博客，我们会分享：\n\n- 孩子们的阅读故事\n- 阅读技巧\n- 家长问答\n\n**欢迎关注！**")
//     .execute(&mut *conn)
//     .await?;
//
//     sqlx::query(
//         "INSERT INTO posts (kind, lang, slug, title, content) VALUES (?, ?, ?, ?, ?)",
//     )
//     .bind("news")
//     .bind("zh")
//     .bind("autumn-courses")
//     .bind("秋季课程开始报名")
//     .bind("## 秋季课程\n\n新学期阅读课程现已开放报名，欢迎五年级同学加入！\n\n名额有限，先到先得。")
//     .execute(&mut *conn)
//     .await?;
//
//     Ok(())
// }
//
// /// Read books/book.txt with the Stage 3 parser and INSERT it as
// /// one books row + one chapters row per chapter.
// async fn seed(conn: &mut SqliteConnection) -> Result<(), sqlx::Error> {
//     let (title, chapters) = load_book();
//
//     let book_id = sqlx::query("INSERT INTO books (title, author) VALUES (?, ?)")
//         .bind(title)
//         .bind("Emma Example")
//         .execute(&mut *conn)
//         .await?
//         .last_insert_rowid(); // the id SQLite just gave our new book
//
//     for (i, (title, paragraphs)) in chapters.iter().enumerate() {
//         sqlx::query(
//             "INSERT INTO chapters (book_id, idx, title, content) VALUES (?, ?, ?, ?)",
//         )
//         .bind(book_id)
//         .bind((i + 1) as i64) // chapters are 1-based
//         .bind(title)
//         .bind(paragraphs.join("\n\n")) // store paragraphs joined by blank lines
//         .execute(&mut *conn) // &mut * = re-borrow, so the loop can use conn again
//         .await?;
//     }
//
//     Ok(())
// }

// ---- The Stage 3 parser, moved here (it's storage's job now) ----

// fn load_book() -> (String, Vec<(String, Vec<String>)>) {
//     let text = std::fs::read_to_string("books/book.txt").unwrap_or_default();
//     parse_book(&text)
// }

/// Our import convention: a line like "Chapter 12" ALL BY ITSELF
/// starts a new chapter. A paragraph that merely STARTS with the
/// word "Chapter" (e.g. "Chapter quizzes are fun.") must NOT count —
/// a bug our unit tests caught before users did.
// fn is_chapter_heading(line: &str) -> bool {
//     line.strip_prefix("Chapter ")
//         .is_some_and(|rest| !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit()))
// }
//
// fn parse_book(text: &str) -> (String, Vec<(String, Vec<String>)>) {
//     let mut chapters = Vec::new();
//     let mut book_title = None;
//     let mut chapter_title = None;
//     let mut paragraphs = Vec::new();
//     let mut current_lines = Vec::new();
//
//     for line in text.lines() {
//         let line = line.trim();
//         if line.is_empty() {
//             if !current_lines.is_empty() {
//                 paragraphs.push(current_lines.join(" "));
//                 current_lines.clear();
//             }
//         } else if is_chapter_heading(line) {
//             if !current_lines.is_empty() {
//                 paragraphs.push(current_lines.join(" "));
//                 current_lines.clear();
//             }
//             if let Some(title) = chapter_title.take() {
//                 chapters.push((title, std::mem::take(&mut paragraphs)));
//             } else if book_title.is_none() {
//                 book_title = paragraphs.first().cloned();
//                 paragraphs.clear();
//             }
//             chapter_title = Some(line.to_string());
//         } else {
//             current_lines.push(line.to_string());
//         }
//     }
//
//     if !current_lines.is_empty() {
//         paragraphs.push(current_lines.join(" "));
//     }
//     if let Some(title) = chapter_title {
//         chapters.push((title, paragraphs));
//     }
//
//     (book_title.unwrap_or_else(|| "Untitled".to_string()), chapters)
// }

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
