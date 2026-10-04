use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{components::A, hooks::use_params_map};
use serde::{Deserialize, Serialize};

pub mod course;
pub mod mindmap;
pub mod middle_east;
pub mod slides;
pub mod videos;

use crate::auth::User;
use self::middle_east::ALL_COURSES;
use self::slides::{SlideDeckViewer, ALL_DECKS};
use self::mindmap::CourseMindMap;
use self::videos::{collect_course_videos, CourseVideoLibrary};
pub use course::{Course, CourseWeek};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ClassComment {
    pub id: i64,
    pub username: String,
    pub content: String,
    pub created_at: String,
}

#[server]
pub async fn get_class_comments(class_id: i64) -> Result<Vec<ClassComment>, ServerFnError> {
    crate::workspace::ssr::require_user().await?;
    let mut conn = crate::db::db().await?;
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
    sqlx::query("INSERT INTO class_comments (class_id, user_id, content) VALUES (?, ?, ?)")
        .bind(class_id).bind(user.id).bind(content).execute(&mut conn).await?;
    Ok(())
}

#[component]
pub fn ClassesPage() -> impl IntoView {
    let user = expect_context::<Resource<Option<User>>>();
    view! {
        <Title text="Classes"/>
        <h2>"Classes"</h2>
        <p>"Class notes and discussions in one place."</p>
<Suspense fallback=|| ().into_view()>

            {move || match user.get() {
                None => ().into_any(),
                Some(None) => view! { <p>"Please " <A href="/login">"log in"</A> " to see your classes."</p> }.into_any(),
                Some(Some(_)) => view! {
                    <ul class="class-list">
                        {ALL_COURSES.iter().map(|c| view! { <li class="class-card">
                            <h3><A href=format!("/classes/{}", c.id)>{c.title}</A></h3>
                            <p>{if c.notes.is_empty() { "No notes yet.".to_string() } else { c.notes.to_string() }}</p>
                            <A attr:class="button" href=format!("/classes/{}", c.id)>"Open class and comments →"</A>
                        </li> }).collect_view()}
                    </ul>
                }.into_any(),
            }}
        </Suspense>
    }
}

#[component]
pub fn ClassDetailPage() -> impl IntoView {
    let params = use_params_map();
    let class_id = move || params.read().get("id").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    let user = expect_context::<Resource<Option<User>>>();
    let add_comment = ServerAction::<AddClassComment>::new();
    let comments = Resource::new(move || (class_id(), add_comment.version().get(), user.get()), move |(id, _, u)| async move { if let Some(Some(_)) = u { get_class_comments(id).await } else { Ok(vec![]) } });

    let active_deck = RwSignal::new(None::<String>);

    view! {
<Suspense fallback=|| view! { <p>"Loading class…"</p> }>

            {move || match user.get() {
                None => ().into_any(),
                Some(None) => view! { <p>"Please " <A href="/login">"log in"</A> " to view this class."</p> }.into_any(),
                Some(Some(_)) => match ALL_COURSES.iter().find(|c| c.id == class_id()) {
                    None => view! { <p>"Class not found."</p> <A href="/classes">"← All classes"</A> }.into_any(),
                    Some(class) => {
                        let course_videos = collect_course_videos(class.slide_decks);
                        let video_count = course_videos.len();
                        let slide_count: usize = ALL_DECKS
                            .iter()
                            .filter(|deck| class.slide_decks.contains(&deck.id))
                            .map(|deck| deck.slide_count)
                            .sum();

                        view! { <div class="course-root">
                        {move || match active_deck.get() {
                            Some(deck_id) => {
                                match ALL_DECKS.iter().find(|d| d.id == deck_id) {
                                    Some(deck) => view! {
                                        <div class="slide-viewer-wrapper">
                                            <div class="slide-viewer-nav">
                                                <button class="slide-viewer-back-btn" on:click=move |_| active_deck.set(None)>
                                                    "← Back to Course Overview"
                                                </button>
                                            </div>
                                            <SlideDeckViewer deck=*deck />
                                        </div>
                                    }.into_any(),
                                    None => view! { <p>"Deck not found"</p> }.into_any()
                                }
                            },
                            None => view! {
                                <div class="course-page">
                                    <Title text=class.title.to_string()/>
                                    
                                    // Hero Section
                                    <div class="course-hero">
                                        <div class="course-hero-inner">
                                            <A href="/classes" attr:class="course-back-link">"← All classes"</A>
                                            <h1 class="course-title">{class.title}</h1>
                                            <p class="course-notes">
                                                {if class.notes.is_empty() { "Deep dive into the history, conflicts, and modern geopolitical landscape.".to_string() } else { class.notes.to_string() }}
                                            </p>
                                            <div class="course-instructor">
                                                <div class="course-instructor-avatar">
                                                    "👨‍🏫"
                                                </div>
                                                <div>
                                                    <div class="course-instructor-name">"Teacher Wang"</div>
                                                    <div class="course-instructor-title">"Lead Instructor"</div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>

                                    <nav class="course-section-nav" aria-label="Course sections">
                                        <a href="#course-videos">"Videos"</a>
                                        <a href="#course-roadmap">"Learning map"</a>
                                        <a href="#course-lessons">"Lesson decks"</a>
                                        <a href="#course-discussion">"Discussion"</a>
                                    </nav>

                                    <div class="course-content-grid">
                                        
                                        <section id="course-videos" class="course-media-section">
                                            <div class="course-section-heading">
                                                <div>
                                                    <p class="course-eyebrow">"WATCH & EXPLORE"</p>
                                                    <h2 class="course-section-title">"Course video library"</h2>
                                                </div>
                                                <span class="course-section-count">{format!("{video_count} clips")}</span>
                                            </div>
                                            <p class="course-section-description">{class.video_intro}</p>
                                            <CourseVideoLibrary videos=course_videos.clone()/>
                                        </section>

                                        <section id="course-roadmap" class="course-roadmap-section">
                                            <div class="course-section-heading">
                                                <div>
                                                    <p class="course-eyebrow">"YOUR LEARNING PATH"</p>
                                                    <h2 class="course-section-title">"Interactive syllabus"</h2>
                                                </div>
                                                <span class="course-section-count">{format!("{} weeks", class.weeks.len())}</span>
                                            </div>
                                            <CourseMindMap course_title=class.title weeks=class.weeks active_deck=active_deck/>
                                        </section>

                                        <section class="course-overview-section">
                                            <div class="course-section-heading">
                                                <div>
                                                    <p class="course-eyebrow">"COURSE OVERVIEW"</p>
                                                    <h2 class="course-section-title">"What you’ll study"</h2>
                                                </div>
                                                <span class="course-section-count">{format!("{slide_count} slides")}</span>
                                            </div>
                                            <details class="course-overview-details">
                                                <summary>"Read the course introduction and syllabus notes"</summary>
                                                <p>{if class.content.is_empty() { "Course details will be added soon.".to_string() } else { class.content.to_string() }}</p>
                                                <p>{if class.syllabus.is_empty() { "A detailed syllabus will be added soon.".to_string() } else { class.syllabus.to_string() }}</p>
                                            </details>
                                        </section>

                                        <section id="course-lessons" class="course-curriculum-section">
                                            <div class="course-section-heading">
                                                <div>
                                                    <p class="course-eyebrow">"GO DEEPER"</p>
                                                    <h2 class="course-section-title">"Interactive lesson decks"</h2>
                                                </div>
                                                <span class="course-section-count">{format!("{} decks", class.slide_decks.len())}</span>
                                            </div>
                                            <div class="curriculum-list">
                                                {class.slide_decks.iter().map(|&deck_id| {
                                                    let d_id = deck_id.to_string();
                                                    if let Some(deck) = ALL_DECKS.iter().find(|d| d.id == deck_id) {
                                                        view! {
                                                            <div class="curriculum-card">
                                                                <div>
                                                                    <h3 class="curriculum-card-title">{deck.title}</h3>
                                                                    <p class="curriculum-card-subtitle">{format!("{} interactive slides", deck.slide_count)}</p>
                                                                </div>
                                                                <button class="curriculum-button" on:click=move |_| active_deck.set(Some(d_id.clone()))>
                                                                    "Open lesson deck →"
                                                                </button>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        ().into_any()
                                                    }
                                                }).collect_view()}
                                            </div>
                                        </section>

                                        // Comments Section
                                        <section id="course-discussion" class="course-discussion-section">
                                            <h2 class="course-discussion-title">"Class Discussion"</h2>
                                            <div class="course-discussion-card">
                                                <ActionForm action=add_comment>
                                                    <input type="hidden" name="class_id" value=move || class_id()/>
                                                    <div class="discussion-form-container">
                                                        <textarea name="content" rows="3" maxlength="2000" required placeholder="Share a question or thought with the class" class="discussion-textarea"></textarea>
                                                        <button type="submit" class="discussion-submit">"Post Comment"</button>
                                                    </div>
                                                </ActionForm>
                                                {move || match add_comment.value().get() {
                                                    Some(Err(ServerFnError::ServerError(msg))) => view! { <p style="color: #ef4444; margin-top: 12px;">{msg}</p> }.into_any(),
                                                    _ => ().into_any(),
                                                }}

                                                <div style="margin-top: 32px;">
                                                    <Transition fallback=|| view! { <p>"Loading discussions…"</p> }>
                                                        {move || match comments.get() {
                                                            None => ().into_any(),
                                                            Some(Err(_)) => view! { <p>"Could not load comments."</p> }.into_any(),
                                                            Some(Ok(rows)) if rows.is_empty() => view! { <p style="color: #6b7280;">"No comments yet. Start the conversation!"</p> }.into_any(),
                                                            Some(Ok(rows)) => view! { 
                                                                <div class="discussion-list">
                                                                    {rows.into_iter().map(|c| view! { 
                                                                        <div class="discussion-item">
                                                                            <div class="discussion-meta">
                                                                                <strong class="discussion-username">{c.username}</strong>
                                                                                <span class="discussion-date">{c.created_at}</span>
                                                                            </div>
                                                                            <p class="discussion-content">{c.content}</p>
                                                                        </div> 
                                                                    }).collect_view()}
                                                                </div> 
                                                            }.into_any(),
                                                        }}
                                                    </Transition>
                                                </div>
                                            </div>
                                        </section>
                                    </div>
                                </div>
                            }.into_any()
                        }} </div>
                    }.into_any()
                },
            }}}
</Suspense>
    }
}
