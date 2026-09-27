// ============================================================
//  posts.rs — blog posts and news.
//  Public side: anyone can LIST and READ posts.
//  Admin side: only role=="admin" can publish or delete.
//  That "check the role on the server" pattern is the whole
//  security model of the members area (you'll see it again
//  in Stages 9-11).
// ============================================================

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// A post in a LIST (no content — lists stay light and fast).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct PostSummary {
    pub slug: String, // the URL name: /blog/hello-world
    pub title: String,
    pub kind: String, // "blog" or "news"
    pub published_at: String,
}

/// A full post page. NOTE: content arrives as RENDERED HTML
/// (the server turned the teacher's Markdown into HTML already).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostPage {
    pub title: String,
    pub kind: String,
    pub content_html: String,
    pub published_at: String,
}

/// List posts. kind = "blog"/"news" filters; "all" = everything (admin).
/// NOTE: we use a plain String instead of Option<String> because
/// Option can't be serialized into a server-function URL — a real
/// gotcha this stage caught (the admin list silently failed).
#[server]
pub async fn get_posts(kind: String) -> Result<Vec<PostSummary>, ServerFnError> {
    let mut conn = crate::db::db().await?;

    let posts = match kind.as_str() {
        "all" => {
            sqlx::query_as::<_, PostSummary>(
                "SELECT slug, title, kind, published_at FROM posts
                 ORDER BY published_at DESC",
            )
            .fetch_all(&mut conn)
            .await?
        }
        k => {
            sqlx::query_as::<_, PostSummary>(
                "SELECT slug, title, kind, published_at FROM posts
                 WHERE kind = ? ORDER BY published_at DESC",
            )
            .bind(k)
            .fetch_all(&mut conn)
            .await?
        }
    };

    Ok(posts)
}

/// The newest 3 posts of any kind (for the home page "Latest" section).
#[server]
pub async fn get_recent_posts() -> Result<Vec<PostSummary>, ServerFnError> {
    let mut conn = crate::db::db().await?;
    let posts = sqlx::query_as::<_, PostSummary>(
        "SELECT slug, title, kind, published_at FROM posts
         ORDER BY published_at DESC LIMIT 3",
    )
    .fetch_all(&mut conn)
    .await?;
    Ok(posts)
}

/// Read one post by its slug.
#[server]
pub async fn get_post(slug: String) -> Result<PostPage, ServerFnError> {
    let mut conn = crate::db::db().await?;

    let row: Option<(String, String, String, String)> = sqlx::query_as(
        "SELECT title, kind, content, published_at FROM posts WHERE slug = ?",
    )
    .bind(&slug)
    .fetch_optional(&mut conn)
    .await?;

    let Some((title, kind, content, published_at)) = row else {
        return Err(ServerFnError::ServerError("post not found".into()));
    };

    Ok(PostPage {
        title,
        kind,
        content_html: ssr::render_markdown(&content),
        published_at,
    })
}

// ----------------------------------------------------------
//  ADMIN-ONLY functions. Security rule: the check happens ON
//  THE SERVER, every time. Hiding a button in the UI is nice
//  but is NOT security — anyone can call a server function
//  directly (remember our curl test in Stage 3!).
// ----------------------------------------------------------

#[server]
pub async fn create_post(
    kind: String,
    slug: String,
    title: String,
    content: String,
) -> Result<(), ServerFnError> {
    let Some(user) = crate::auth::ssr::current_user().await? else {
        return Err(ServerFnError::ServerError("Please log in.".into()));
    };
    if user.role != "admin" {
        return Err(ServerFnError::ServerError("This is for the teacher (admin) only.".into()));
    }

    let slug = slug.trim().to_lowercase();
    let slug_ok = !slug.is_empty()
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if !slug_ok {
        return Err(ServerFnError::ServerError(
            "Slug: lowercase letters, numbers and dashes only.".into(),
        ));
    }
    if kind != "blog" && kind != "news" {
        return Err(ServerFnError::ServerError("Type must be blog or news.".into()));
    }
    if title.trim().is_empty() || content.trim().is_empty() {
        return Err(ServerFnError::ServerError("Title and content can't be empty.".into()));
    }

    let mut conn = crate::db::db().await?;
    let result = sqlx::query("INSERT INTO posts (kind, slug, title, content) VALUES (?, ?, ?, ?)")
        .bind(&kind)
        .bind(&slug)
        .bind(title.trim())
        .bind(&content)
        .execute(&mut conn)
        .await;

    match result {
        Ok(_) => {
            tracing::info!(event = "publish", user = %user.username, kind = %kind, slug = %slug, "post published");
            Ok(())
        }
        Err(_) => Err(ServerFnError::ServerError("That slug is already used.".into())),
    }
}

#[server]
pub async fn delete_post(slug: String) -> Result<(), ServerFnError> {
    let Some(user) = crate::auth::ssr::current_user().await? else {
        return Err(ServerFnError::ServerError("Please log in.".into()));
    };
    if user.role != "admin" {
        return Err(ServerFnError::ServerError("This is for the teacher (admin) only.".into()));
    }

    let mut conn = crate::db::db().await?;
    sqlx::query("DELETE FROM posts WHERE slug = ?")
        .bind(&slug)
        .execute(&mut conn)
        .await?;
    tracing::info!(event = "delete_post", user = %user.username, slug = %slug, "post deleted");
    Ok(())
}

// ----------------------------------------------------------
#[cfg(feature = "ssr")]
pub mod ssr {
    /// Turn the teacher's Markdown into HTML.
    /// Markdown is much easier to write than HTML by hand:
    ///   ## Heading   **bold**   - list item   [text](https://link)
    pub fn render_markdown(md: &str) -> String {
        use pulldown_cmark::{html, Parser};
        let mut out = String::new();
        html::push_html(&mut out, Parser::new(md));
        out
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    #[test]
    fn markdown_becomes_html() {
        let html = super::ssr::render_markdown("# Hello\n\nsome **bold** text");
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }
}
