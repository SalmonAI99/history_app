// ============================================================
//  workspace.rs — the user's own stuff (Stage 9).
//  Reading progress, chapter notes, personal writings.
//  THE ONE RULE of this module: every query is scoped by
//  user_id from the SESSION (never from the client).
//  Leaking one user's note to another is the unforgivable bug.
// ============================================================

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

use crate::auth::User;

/// One note or writing, as shown in the workspace list.
/// Notes (kind="note") link back to their book chapter;
/// writings (kind="writing") just have a title.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct DocumentSummary {
    pub id: i64,
    pub kind: String,
    pub title: String,
    pub content: String,
    pub book_id: Option<i64>,
    pub book_title: Option<String>, // filled by a LEFT JOIN (notes only)
    pub chapter: Option<i64>,       // notes only
    pub created_at: String,
}

// ----------------------------------------------------------
//  SERVER FUNCTIONS
// ----------------------------------------------------------

/// Remember where this user stopped reading a book.
/// UPSERT = insert, or update if a row already exists for
/// (user_id, book_id) — one progress row per user per book.
#[server]
pub async fn save_progress(book_id: i64, chapter: i64) -> Result<(), ServerFnError> {
    let user = ssr::require_user().await?;
    let mut conn = crate::db::db().await?;

    sqlx::query(
        "INSERT INTO progress (user_id, book_id, chapter) VALUES (?, ?, ?)
         ON CONFLICT (user_id, book_id) DO UPDATE SET chapter = excluded.chapter",
    )
    .bind(user.id)
    .bind(book_id)
    .bind(chapter)
    .execute(&mut conn)
    .await?;

    tracing::info!(event = "progress", user = %user.username, book_id, chapter, "progress saved");
    Ok(())
}

/// Save a note about the chapter the user is reading.
#[server]
pub async fn save_note(book_id: i64, chapter: i64, content: String) -> Result<(), ServerFnError> {
    let user = ssr::require_user().await?;

    let content = content.trim().to_string();
    if content.is_empty() {
        return Err(ServerFnError::ServerError("Write something first.".into()));
    }

    let mut conn = crate::db::db().await?;
    sqlx::query(
        "INSERT INTO documents (user_id, kind, title, content, book_id, chapter)
         VALUES (?, 'note', '', ?, ?, ?)",
    )
    .bind(user.id)
    .bind(&content)
    .bind(book_id)
    .bind(chapter)
    .execute(&mut conn)
    .await?;

    tracing::info!(event = "save_note", user = %user.username, book_id, chapter, "note saved");
    Ok(())
}

/// Save a standalone writing (not tied to any book).
#[server]
pub async fn save_writing(title: String, content: String) -> Result<(), ServerFnError> {
    let user = ssr::require_user().await?;

    let title = title.trim().to_string();
    let content = content.trim().to_string();
    if title.is_empty() || content.is_empty() {
        return Err(ServerFnError::ServerError("Title and content can't be empty.".into()));
    }

    let mut conn = crate::db::db().await?;
    sqlx::query("INSERT INTO documents (user_id, kind, title, content) VALUES (?, 'writing', ?, ?)")
        .bind(user.id)
        .bind(&title)
        .bind(&content)
        .execute(&mut conn)
        .await?;

    tracing::info!(event = "save_writing", user = %user.username, "writing saved");
    Ok(())
}

/// Everything this user has written — notes AND writings,
/// newest first. The LEFT JOIN fetches the book title for notes.
#[server]
pub async fn get_my_documents() -> Result<Vec<DocumentSummary>, ServerFnError> {
    let user = ssr::require_user().await?;
    let mut conn = crate::db::db().await?;

    let docs = sqlx::query_as::<_, DocumentSummary>(
        "SELECT d.id, d.kind, d.title, d.content, d.book_id,
                b.title AS book_title, d.chapter, d.created_at
         FROM documents d
         LEFT JOIN books b ON b.id = d.book_id
         WHERE d.user_id = ?
         ORDER BY d.created_at DESC",
    )
    .bind(user.id)
    .fetch_all(&mut conn)
    .await?;

    Ok(docs)
}

/// Delete one of the user's documents.
/// Ownership is enforced IN THE QUERY (WHERE id = ? AND user_id = ?):
/// you can only ever delete your own stuff. Never trust the UI.
#[server]
pub async fn delete_document(id: i64) -> Result<(), ServerFnError> {
    let user = ssr::require_user().await?;
    let mut conn = crate::db::db().await?;

    sqlx::query("DELETE FROM documents WHERE id = ? AND user_id = ?")
        .bind(id)
        .bind(user.id)
        .execute(&mut conn)
        .await?;

    tracing::info!(event = "delete_doc", user = %user.username, doc_id = id, "document deleted");
    Ok(())
}

// ----------------------------------------------------------
#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::auth::User;
    use leptos::prelude::*;

    /// The first line of every user-data function: WHO is asking?
    /// Fails with "Please log in." if there's no valid session.
    pub async fn require_user() -> Result<User, ServerFnError> {
        let Some(user) = crate::auth::ssr::current_user().await? else {
            return Err(ServerFnError::ServerError("Please log in.".into()));
        };
        Ok(user)
    }
}

// ----------------------------------------------------------
//  PAGE: /workspace — your notes & writings in one place.
// ----------------------------------------------------------

#[component]
pub fn WorkspacePage() -> impl IntoView {
    let user = expect_context::<Resource<Option<User>>>();
    let save_writing = ServerAction::<SaveWriting>::new();
    let delete_action = ServerAction::<DeleteDocument>::new();

    // After saving or deleting, the list refetches itself.
    let docs = Resource::new(
        move || (save_writing.version().get(), delete_action.version().get()),
        |_| get_my_documents(),
    );

    view! {
        <Transition fallback=|| ()>
            {move || match user.get() {
                None => ().into_any(),

                Some(None) => view! {
                    <p>"Please " <A href="/login">"log in"</A> " to see your workspace."</p>
                }.into_any(),

                Some(Some(_)) => view! {
                    <Title text="My Workspace"/>
                    <h2>"My Workspace"</h2>

                    <section class="post-form">
                        <h3>"New writing"</h3>
                        <ActionForm action=save_writing>
                            <input type="text" name="title" placeholder="Title" required/>
                            <textarea name="content" rows=4 placeholder="Write anything you like..." required></textarea>
                            <button type="submit">"Save"</button>
                        </ActionForm>
                        {move || match save_writing.value().get() {
                            Some(Ok(())) => view! { <p class="ok">"Saved!"</p> }.into_any(),
                            Some(Err(ServerFnError::ServerError(msg))) => {
                                view! { <p class="error">{msg}</p> }.into_any()
                            }
                            _ => ().into_any(),
                        }}
                    </section>

                    <h3>"My notes & writings"</h3>
                    <Transition fallback=move || view! { <p class="loading">"Loading..."</p> }>
                        {move || match docs.get() {
                            None => ().into_any(),
                            Some(Err(_)) => view! {
                                <p>"Could not load your documents."</p>
                            }.into_any(),
                            Some(Ok(docs)) => {
                                if docs.is_empty() {
                                    view! {
                                        <p>"Nothing yet — open a book, save a note on a chapter, or write something above!"</p>
                                    }.into_any()
                                } else {
                                    view! {
                                        <ul class="doc-list">
                                            {docs.into_iter().map(|d| {
                                                // Notes get a link back to their chapter;
                                                // writings show their title.
                                                let heading = if d.kind == "note" {
                                                    match (d.book_id, d.chapter) {
                                                        (Some(b), Some(c)) => view! {
                                                            <A href=format!("/book/{b}/chapter/{c}")>
                                                                {format!(
                                                                    "Note on {} — Chapter {c}",
                                                                    d.book_title.clone().unwrap_or_default()
                                                                )}
                                                            </A>
                                                        }.into_any(),
                                                        _ => ().into_any(),
                                                    }
                                                } else {
                                                    view! { <strong>{d.title.clone()}</strong> }.into_any()
                                                };

                                                view! {
                                                    <li class="doc-item">
                                                        <div class="doc-heading">
                                                            {heading}
                                                            <span class="date">{d.created_at.clone()}</span>
                                                        </div>
                                                        <p class="doc-content">{d.content.clone()}</p>
                                                        <ActionForm action=delete_action>
                                                            <input type="hidden" name="id" value=d.id/>
                                                            <button type="submit">"Delete"</button>
                                                        </ActionForm>
                                                    </li>
                                                }
                                            }).collect_view()}
                                        </ul>
                                    }.into_any()
                                }
                            },
                        }}
                    </Transition>
                }.into_any(),
            }}
        </Transition>
    }
}
