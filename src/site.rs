// Public home and teacher pages, blog/news listings and details, and the admin page.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::posts::{get_post, get_posts, get_recent_posts};

// ----------------------------------------------------------
//  HOME — the marketing front door.
// ----------------------------------------------------------

#[component]
pub fn HomePage() -> impl IntoView {
    let posts = get_recent_posts();

    view! {
        <Title text="星阅读 StarRead — 儿童阅读课程"/>


        <section class="hero">
        <video
            class="hero-video"
            autoplay
            muted
            loop
            playsinline
            preload="auto"
        >
            <source src="editorial_MASTER_1791176464742.mp4" type="video/mp4"/>
        </video>

        <div class="hero-content">
            <h1>Explore History</h1>
            <p>Travel through time and discover the past.</p>
        </div>
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
                <h3>"📚 Read at your pace"</h3>
                <p>"Explore stories chapter by chapter in a focused reading view."</p>
            </div>
        </section>

        <section>
            <h2>"Latest"</h2>
            <ul class="post-list">
                {posts.into_iter().map(|p| view! {
                    <li>
                        <span class="tag">{p.kind}</span>
                        <A href=format!("/{}/{}", p.kind, p.slug)>{p.title.to_string()}</A>
                        <span class="date">{p.published_at}</span>
                    </li>
                }).collect_view()}
            </ul>
        </section>
    }
}

// ----------------------------------------------------------
//  TEACHER — static info page
// ----------------------------------------------------------

#[component]
pub fn TeacherPage() -> impl IntoView {
    view! {
        <Title text="About the Teacher"/>
        <h2>"About the Teacher"</h2>
        <div class="teacher-card">
            <p>
                "Teacher Wang has helped children fall in love with reading for 15 years. \
                 She believes every child is one good book away from becoming a reader."
            </p>
        </div>
    }
}

// ----------------------------------------------------------
//  BLOG & NEWS — one shared list component, two flavors.
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
    let posts = get_posts(kind);

    view! {
        <Title text=heading/>
        <h2>{heading}</h2>
        {
            if posts.is_empty() {
                view! { <p>"Nothing here yet — check back soon!"</p> }.into_any()
            } else {
                view! {
                    <ul class="post-list">
                        {posts.into_iter().map(|p| view! {
                            <li>
                                <A href=format!("/{}/{}", p.kind, p.slug)>{p.title.to_string()}</A>
                                <span class="date">{p.published_at.to_string()}</span>
                            </li>
                        }).collect_view()}
                    </ul>
                }.into_any()
            }
        }
    }
}

// ----------------------------------------------------------
//  ONE POST — rendered directly from the Leptos view
// ----------------------------------------------------------

#[component]
pub fn PostDetailPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();
    
    view! {
        {move || match get_post(&slug()) {
            None => view! {
                <p>"This post doesn't exist (anymore)."</p>
                <p><A href="/">"← Back home"</A></p>
            }.into_any(),
            Some(p) => view! {
                <Title text=p.title.to_string()/>
                <article class="post">
                    <h2>{p.title}</h2>
                    <p class="date">{p.published_at}</p>
                    <div class="post-content">
                        {(p.render)()}
                    </div>
                </article>
            }.into_any(),
        }}
    }
}

// ----------------------------------------------------------
//  ADMIN
// ----------------------------------------------------------

#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <Title text="Admin"/>
        <h2>"Admin"</h2>
        <p>"Posts are now managed via code in " <code>"src/posts.rs"</code> "."</p>
        <p>"To create a new post, write a new Leptos component and add it to " <code>"ALL_POSTS"</code> " in " <code>"src/posts.rs"</code> "."</p>
    }
}
