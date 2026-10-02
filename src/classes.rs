use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{components::A, hooks::use_params_map};
use serde::{Deserialize, Serialize};

use crate::auth::User;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ClassSummary {
    pub id: i64,
    pub title: String,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ClassComment {
    pub id: i64,
    pub username: String,
    pub content: String,
    pub created_at: String,
}

#[server]
pub async fn get_classes() -> Result<Vec<ClassSummary>, ServerFnError> {
    crate::workspace::ssr::require_user().await?;
    let mut conn = crate::db::db().await?;
    Ok(sqlx::query_as("SELECT id, title, notes FROM classes ORDER BY title COLLATE NOCASE")
        .fetch_all(&mut conn).await?)
}

#[server]
pub async fn create_class(title: String, notes: String) -> Result<(), ServerFnError> {
    let user = crate::workspace::ssr::require_user().await?;
    if user.role != "admin" { return Err(ServerFnError::ServerError("Only the teacher can add classes.".into())); }
    let title = title.trim();
    if title.is_empty() || title.len() > 120 || notes.len() > 20_000 {
        return Err(ServerFnError::ServerError("Enter a class name (up to 120 characters) and notes under 20,000 characters.".into()));
    }
    let mut conn = crate::db::db().await?;
    sqlx::query("INSERT INTO classes (title, notes) VALUES (?, ?)").bind(title).bind(notes.trim()).execute(&mut conn).await?;
    Ok(())
}

#[server]
pub async fn get_class_comments(class_id: i64) -> Result<Vec<ClassComment>, ServerFnError> {
    crate::workspace::ssr::require_user().await?;
    let mut conn = crate::db::db().await?;
    let exists: Option<(i64,)> = sqlx::query_as("SELECT id FROM classes WHERE id = ?").bind(class_id).fetch_optional(&mut conn).await?;
    if exists.is_none() { return Err(ServerFnError::ServerError("Class not found.".into())); }
    Ok(sqlx::query_as("SELECT cc.id, u.username, cc.content, cc.created_at FROM class_comments cc JOIN users u ON u.id = cc.user_id WHERE cc.class_id = ? ORDER BY cc.created_at, cc.id")
        .bind(class_id).fetch_all(&mut conn).await?)
}

#[server]
pub async fn add_class_comment(class_id: i64, content: String) -> Result<(), ServerFnError> {
    let user = crate::workspace::ssr::require_user().await?;
    let content = content.trim();
    if content.is_empty() || content.len() > 2000 {
        return Err(ServerFnError::ServerError("Comment must be 1–2,000 characters.".into()));
    }
    let mut conn = crate::db::db().await?;
    let result = sqlx::query("INSERT INTO class_comments (class_id, user_id, content) SELECT id, ?, ? FROM classes WHERE id = ?")
        .bind(user.id).bind(content).bind(class_id).execute(&mut conn).await?;
    if result.rows_affected() == 0 { return Err(ServerFnError::ServerError("Class not found.".into())); }
    Ok(())
}

#[component]
pub fn ClassesPage() -> impl IntoView {
    let user = expect_context::<Resource<Option<User>>>();
    let create = ServerAction::<CreateClass>::new();
    let classes = Resource::new(move || create.version().get(), |_| get_classes());
    view! {
        <Title text="Classes"/>
        <h2>"Classes"</h2>
        <p>"Class notes and discussions in one place."</p>
        <Transition fallback=|| ().into_view()>
            {move || match user.get() {
                None => ().into_any(),
                Some(None) => view! { <p>"Please " <A href="/login">"log in"</A> " to see your classes."</p> }.into_any(),
                Some(Some(u)) => view! {
                    {if u.role == "admin" { view! {
                        <section class="post-form class-create">
                            <h3>"Add a class"</h3>
                            <ActionForm action=create>
                                <label>"Class name" <input name="title" maxlength="120" required/></label>
                                <label>"Class notes" <textarea name="notes" rows="5" placeholder="Share notes and class information"></textarea></label>
                                <button type="submit">"Create class"</button>
                            </ActionForm>
                            {move || match create.value().get() {
                                Some(Ok(())) => view! { <p class="ok">"Class added."</p> }.into_any(),
                                Some(Err(ServerFnError::ServerError(msg))) => view! { <p class="error">{msg}</p> }.into_any(),
                                _ => ().into_any(),
                            }}
                        </section>
                    }.into_any() } else { ().into_any() }}
                    <Transition fallback=|| view! { <p>"Loading classes…"</p> }>
                        {move || match classes.get() {
                            None => ().into_any(),
                            Some(Err(_)) => view! { <p>"Could not load classes. Please log in again."</p> }.into_any(),
                            Some(Ok(items)) if items.is_empty() => view! { <p>"No classes yet."</p> }.into_any(),
                            Some(Ok(items)) => view! { <ul class="class-list">
                                {items.into_iter().map(|c| view! { <li class="class-card">
                                    <h3><A href=format!("/classes/{}", c.id)>{c.title}</A></h3>
                                    <p>{if c.notes.is_empty() { "No notes yet.".to_string() } else { c.notes.clone() }}</p>
                                    <A attr:class="button" href=format!("/classes/{}", c.id)>"Open class and comments →"</A>
                                </li> }).collect_view()}
                            </ul> }.into_any(),
                        }}
                    </Transition>
                }.into_any(),
            }}
        </Transition>
    }
}

#[component]
pub fn ClassDetailPage() -> impl IntoView {
    let params = use_params_map();
    let class_id = move || params.read().get("id").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    let classes = Resource::new(|| (), |_| get_classes());
    let add_comment = ServerAction::<AddClassComment>::new();
    let comments = Resource::new(move || (class_id(), add_comment.version().get()), |(id, _)| get_class_comments(id));
    view! {
        <Transition fallback=|| view! { <p>"Loading class…"</p> }>
            {move || match classes.get() {
                None => ().into_any(),
                Some(Err(_)) => view! { <p>"Please " <A href="/login">"log in"</A> " to view this class."</p> }.into_any(),
                Some(Ok(items)) => match items.into_iter().find(|c| c.id == class_id()) {
                    None => view! { <p>"Class not found."</p> <A href="/classes">"← All classes"</A> }.into_any(),
                    Some(class) => view! {
                        <Title text=class.title.clone()/>
                        <p><A href="/classes">"← All classes"</A></p>
                        <article class="class-detail">
                            <h2>{class.title}</h2>
                            <h3>"Class notes"</h3>
                            <p class="class-notes">{if class.notes.is_empty() { "No notes have been added yet.".to_string() } else { class.notes }}</p>
                        </article>
                        <section class="class-comments">
                            <h3>"Class comments"</h3>
                            <ActionForm action=add_comment>
                                <input type="hidden" name="class_id" value=class_id()/>
                                <label>"Leave a comment" <textarea name="content" rows="3" maxlength="2000" required placeholder="Share a question or thought with the class"></textarea></label>
                                <button type="submit">"Post comment"</button>
                            </ActionForm>
                            {move || match add_comment.value().get() {
                                Some(Err(ServerFnError::ServerError(msg))) => view! { <p class="error">{msg}</p> }.into_any(),
                                _ => ().into_any(),
                            }}
                            <Transition fallback=|| view! { <p>"Loading comments…"</p> }>
                                {move || match comments.get() {
                                    None => ().into_any(),
                                    Some(Err(_)) => view! { <p>"Could not load comments."</p> }.into_any(),
                                    Some(Ok(rows)) if rows.is_empty() => view! { <p>"No comments yet. Start the conversation!"</p> }.into_any(),
                                    Some(Ok(rows)) => view! { <ul class="comment-list">
                                        {rows.into_iter().map(|c| view! { <li>
                                            <div class="comment-meta"><strong>{c.username}</strong><span class="date">{c.created_at}</span></div>
                                            <p>{c.content}</p>
                                        </li> }).collect_view()}
                                    </ul> }.into_any(),
                                }}
                            </Transition>
                        </section>
                    }.into_any(),
                },
            }}
        </Transition>
    }
}
