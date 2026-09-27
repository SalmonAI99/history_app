// ============================================================
//  site.rs — the PUBLIC marketing pages + the admin publishing page.
//    /         -> the pitch (home)
//    /teacher  -> about the teacher
//    /blog     -> post list          /blog/<slug> -> one post
//    /news     -> news list          /news/<slug> -> one post
//    /admin    -> publish & delete posts (admin only)
// ============================================================

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::auth::User;
use crate::posts::{get_post, get_posts, get_recent_posts, CreatePost, DeletePost};

// ----------------------------------------------------------
//  HOME — the marketing front door.
//  EDIT ME: replace the placeholder copy with your real pitch
//  (and in Chinese for your audience!).
// ----------------------------------------------------------

#[component]
pub fn HomePage() -> impl IntoView {
    let posts = Resource::new(|| (), |_| get_recent_posts());

    view! {
        <Title text="星阅读 StarRead — 儿童阅读课程"/>

        <section class="hero">
            // EDIT ME: your real headline + pitch
            <h2>"Give your child the magic of reading"</h2>
            <p>"A guided reading course for 5th-grade students — stories they love, skills for life."</p>
            <p>
                <A attr:class="button" href="/library">"Start reading →"</A>
            </p>
        </section>

        <section class="features">
            <div class="card">
                <h3>"📖 Real books"</h3>
                <p>"Carefully chosen chapter books at the right level."</p>
            </div>
            <div class="card">
                <h3>"👩‍🏫 Real teacher"</h3>
                <p>"Guided by an experienced teacher, not just an app."</p>
            </div>
            <div class="card">
                <h3>"✍️ Real progress"</h3>
                <p>"Children save progress and write their own thoughts."</p>
            </div>
        </section>

        <section>
            <h2>"Latest"</h2>
            <Transition fallback=move || view! { <p class="loading">"Loading..."</p> }>
                {move || match posts.get() {
                    None => ().into_any(),
                    Some(Err(_)) => ().into_any(),
                    Some(Ok(posts)) => view! {
                        <ul class="post-list">
                            {posts.into_iter().map(|p| view! {
                                <li>
                                    <span class="tag">{p.kind.clone()}</span>
                                    <A href=format!("/{}/{}", p.kind, p.slug)>{p.title.clone()}</A>
                                    <span class="date">{p.published_at.clone()}</span>
                                </li>
                            }).collect_view()}
                        </ul>
                    }.into_any(),
                }}
            </Transition>
        </section>
    }
}

// ----------------------------------------------------------
//  TEACHER — static info page (write your real bio here).
// ----------------------------------------------------------

#[component]
pub fn TeacherPage() -> impl IntoView {
    view! {
        <Title text="About the Teacher"/>
        <h2>"About the Teacher"</h2>
        <div class="teacher-card">
            // EDIT ME: your real name, photo, story, and qualifications.
            <p>
                "Teacher Wang has helped children fall in love with reading for 15 years. \
                 She believes every child is one good book away from becoming a reader."
            </p>
        </div>
    }
}

// ----------------------------------------------------------
//  BLOG & NEWS — one shared list component, two flavors.
//  (Components with props = reusable pages!)
// ----------------------------------------------------------

#[component]
pub fn BlogPage() -> impl IntoView {
    view! { <PostList kind="blog" heading="Blog"/> }
}

#[component]
pub fn NewsPage() -> impl IntoView {
    view! { <PostList kind="news" heading="News"/> }
}

#[component]
fn PostList(kind: &'static str, heading: &'static str) -> impl IntoView {
    let posts = Resource::new(|| (), move |_| get_posts(kind.to_string()));

    view! {
        <Title text=heading/>
        <h2>{heading}</h2>
        <Transition fallback=move || view! { <p class="loading">"Loading..."</p> }>
            {move || match posts.get() {
                None => ().into_any(),
                Some(Err(_)) => view! { <p>"Could not load posts."</p> }.into_any(),
                Some(Ok(posts)) => {
                    if posts.is_empty() {
                        view! { <p>"Nothing here yet — check back soon!"</p> }.into_any()
                    } else {
                        view! {
                            <ul class="post-list">
                                {posts.into_iter().map(|p| view! {
                                    <li>
                                        <A href=format!("/{kind}/{}", p.slug)>{p.title.clone()}</A>
                                        <span class="date">{p.published_at.clone()}</span>
                                    </li>
                                }).collect_view()}
                            </ul>
                        }.into_any()
                    }
                },
            }}
        </Transition>
    }
}

// ----------------------------------------------------------
//  ONE POST — rendered from the teacher's Markdown.
// ----------------------------------------------------------

#[component]
pub fn PostDetailPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();
    let post = Resource::new(move || slug(), move |s| get_post(s));

    view! {
        <Transition fallback=move || view! { <p class="loading">"Loading..."</p> }>
            {move || match post.get() {
                None => ().into_any(),
                Some(Err(_)) => view! {
                    <p>"This post doesn't exist (anymore)."</p>
                    <p><A href="/">"← Back home"</A></p>
                }.into_any(),
                Some(Ok(p)) => view! {
                    <Title text=p.title.clone()/>
                    <article class="post">
                        <h2>{p.title}</h2>
                        <p class="date">{p.published_at}</p>
                        // inner_html injects raw HTML — normally an XSS danger,
                        // which is why this is OK ONLY because the author is
                        // the trusted admin. NEVER do this with user input!
                        <div class="post-content" inner_html=p.content_html></div>
                    </article>
                }.into_any(),
            }}
        </Transition>
    }
}

// ----------------------------------------------------------
//  ADMIN — publish new posts, delete old ones.
//  Two gates: the UI hides it from non-admins (below), AND the
//  server functions check the role again (see posts.rs). Both!
// ----------------------------------------------------------

#[component]
pub fn AdminPage() -> impl IntoView {
    let user = expect_context::<Resource<Option<User>>>();
    let create = ServerAction::<CreatePost>::new();
    let delete_action = ServerAction::<DeletePost>::new();

    // After publishing/deleting, the version number changes and the
    // Resource refetches — the list updates itself. Reactivity again!
    let posts = Resource::new(
        move || (create.version().get(), delete_action.version().get()),
        |_| get_posts("all".to_string()),
    );

    view! {
        <Transition fallback=|| ()>
            {move || match user.get() {
                None => ().into_any(),

                Some(None) => view! {
                    <p>"Please " <A href="/login">"log in"</A> " first."</p>
                }.into_any(),

                Some(Some(u)) if u.role != "admin" => view! {
                    <p>"This page is for the teacher (admin) only."</p>
                }.into_any(),

                Some(Some(_admin)) => view! {
                    <Title text="Admin — publish a post"/>
                    <h2>"Publish a post"</h2>

                    <ActionForm action=create attr:class="post-form">
                        <label>"Type"
                            <select name="kind">
                                <option value="blog">"Blog"</option>
                                <option value="news">"News"</option>
                            </select>
                        </label>
                        <label>"URL name (slug): lowercase letters, numbers, dashes"
                            <input type="text" name="slug" required placeholder="my-first-post"/>
                        </label>
                        <label>"Title"
                            <input type="text" name="title" required/>
                        </label>
                        <label>"Content — Markdown: ## heading, **bold**, - lists"
                            <textarea name="content" rows="8" required></textarea>
                        </label>
                        <button type="submit">"Publish"</button>
                    </ActionForm>

                    {move || match create.value().get() {
                        Some(Ok(())) => view! { <p class="ok">"Published!"</p> }.into_any(),
                        Some(Err(ServerFnError::ServerError(msg))) => {
                            view! { <p class="error">{msg}</p> }.into_any()
                        }
                        _ => ().into_any(),
                    }}

                    <h3>"Existing posts"</h3>
                    <Transition fallback=move || view! { <p class="loading">"Loading..."</p> }>
                        {move || match posts.get() {
                            None => ().into_any(),
                            Some(Err(_)) => ().into_any(),
                            Some(Ok(posts)) => view! {
                                <ul class="post-list admin-list">
                                    {posts.into_iter().map(|p| view! {
                                        <li>
                                            <span class="tag">{p.kind.clone()}</span>
                                            <A href=format!("/{}/{}", p.kind, p.slug)>{p.title.clone()}</A>
                                            <ActionForm action=delete_action>
                                                <input type="hidden" name="slug" value=p.slug/>
                                                <button type="submit">"Delete"</button>
                                            </ActionForm>
                                        </li>
                                    }).collect_view()}
                                </ul>
                            }.into_any(),
                        }}
                    </Transition>
                }.into_any(),
            }}
        </Transition>
    }
}
