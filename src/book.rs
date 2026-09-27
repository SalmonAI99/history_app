// ============================================================
//  book.rs — the book API: shared structs + server functions.
//  Stage 5: the data now comes from the DATABASE (see db.rs).
//  Notice what did NOT change: the function names and the
//  structs — so the UI in app.rs barely changes. Layers!
// ============================================================

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// One book in the library list.
/// FromRow = "SQLx can build this struct from a database row" (server-only).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct BookInfo {
    pub id: i64, // SQLite INTEGER = i64 in Rust
    pub title: String,
}

/// What the browser receives when it asks for a chapter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChapterPage {
    pub title: String,
    pub paragraphs: Vec<String>,
    pub total_chapters: i64,
}

/// List all books in the library.
#[server]
pub async fn get_books() -> Result<Vec<BookInfo>, ServerFnError> {
    let mut conn = crate::db::db().await?;

    let books = sqlx::query_as::<_, BookInfo>("SELECT id, title FROM books ORDER BY id")
        .fetch_all(&mut conn)
        .await?;

    Ok(books)
}

/// Get one chapter of a book (chapters are 1-based, see Stage 4).
#[server]
pub async fn get_chapter(book_id: i64, chapter: i64) -> Result<ChapterPage, ServerFnError> {
    if chapter < 1 {
        return Err(ServerFnError::ServerError("chapters start at 1".into()));
    }

    let mut conn = crate::db::db().await?;

    // The "?"s are PLACEHOLDERS: values travel separately via .bind().
    // NEVER build SQL with format!() and user input — that's SQL injection,
    // the classic way apps get hacked. Placeholders make it impossible.
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT title, content FROM chapters WHERE book_id = ? AND idx = ?",
    )
    .bind(book_id)
    .bind(chapter)
    .fetch_optional(&mut conn)
    .await?;

    let Some((title, content)) = row else {
        return Err(ServerFnError::ServerError(format!("no chapter {chapter}")));
    };

    let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chapters WHERE book_id = ?")
        .bind(book_id)
        .fetch_one(&mut conn)
        .await?;

    Ok(ChapterPage {
        title,
        // paragraphs are stored joined by blank lines — split them back
        paragraphs: content.split("\n\n").map(|s| s.to_string()).collect(),
        total_chapters: total,
    })
}
