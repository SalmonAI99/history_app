// Root Leptos UI: document shell, shared user state, navigation, and route table.

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes, A},
    ParamSegment, StaticSegment,
};
use crate::auth::{get_user, Logout, User};
// use crate::book::{get_books, get_chapter};
use crate::search;
use crate::site;
use crate::login::LoginPage;

/// `shell` is the HTML "frame" around your app.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="zh-CN">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Ask ONCE "who is logged in?" and share the answer app-wide.
    // Any component can now do: expect_context::<Resource<Option<User>>>()
    let user = Resource::new(|| (), |_| async { get_user().await.unwrap_or(None) });
    provide_context(user);

    view! {
        <Stylesheet id="leptos" href="/pkg/bookreader.css"/>
        <Title text="阅读之家"/>

        <Router>
            <header class="site-header">
                <h1 class="site-brand">
                    <A href="/">
                        <svg viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                            <path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H20v20H6.5a2.5 2.5 0 0 1 0-5H20"/>
                        </svg>
                        <span>"阅读之家"</span>
                    </A>
                </h1>
                // Site navigation (the public pages from site.rs):
                <nav class="site-nav">
                    <A href="/classes">"课程"</A>
                    <A href="/teacher">"老师"</A>
                    <A href="/blog">"文章"</A>
                    <A href="/news">"活动"</A>
                </nav>
                <form class="global-search" action="/search" method="get" role="search">
                    <label class="visually-hidden" for="global-search-input">"搜索全站内容"</label>
                    <input id="global-search-input" type="search" name="q" placeholder="搜索书籍、课程和文章…"/>
                    <button type="submit">"搜索"</button>
                </form>
                <Nav/>
            </header>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    // PUBLIC zone (marketing) — see site.rs
                    <Route path=StaticSegment("") view=site::HomePage/>
                    // <Route path=StaticSegment("library") view=LibraryPage/>
                    <Route path=StaticSegment("search") view=search::SearchPage/>
                    // <Route path=(StaticSegment("book"), ParamSegment("id"), StaticSegment("chapter"), ParamSegment("chapter")) view=ReaderPage/>
                    // <Route path=StaticSegment("materials") view=files::MaterialsPage/>
                    // <Route path=StaticSegment("member") view=files::VideosPage/>
                    // <Route path=StaticSegment("slides") view=classes::slides::SlidesIndexPage/>
                    // <Route path=(StaticSegment("slides"), ParamSegment("id")) view=classes::slides::SlideDeckPage/>
                    <Route path=StaticSegment("teacher") view=site::TeacherPage/>
                    <Route path=StaticSegment("classes") view=crate::classes::view::ClassesPage/>
                    <Route path=(StaticSegment("classes"), ParamSegment("id")) view=crate::classes::view::ClassDetailPage/>
                    <Route path=StaticSegment("profile") view=crate::classes::view::ProfilePage/>
                    <Route path=StaticSegment("blog") view=site::BlogPage/>
                    <Route path=(StaticSegment("blog"), ParamSegment("slug")) view=site::PostDetailPage/>
                    <Route path=StaticSegment("news") view=site::NewsPage/>
                    <Route path=(StaticSegment("news"), ParamSegment("slug")) view=site::PostDetailPage/>

                    // accounts
                    <Route path=StaticSegment("login") view=LoginPage/>
                    <Route path=StaticSegment("admin") view=site::AdminPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// The right side of the header: "Log in" — or your name + "Log out".
#[component]
fn Nav() -> impl IntoView {
    let user = expect_context::<Resource<Option<User>>>();
    let logout = ServerAction::<Logout>::new();

    // An Effect = "do something when these signals change".
    // Here: when logout succeeds, reload the user so the nav
    // flips back to "Log in".
    Effect::new(move |_| {
        if matches!(logout.value().get(), Some(Ok(()))) {
            user.refetch();
        }
    });

    view! {
        <nav class="user-nav">
            <Suspense fallback=|| ()>
                // Suspend + .await (not .get()) so this boundary always sees the
                // user resource resolve; with .get() SSR could intermittently
                // never close this Suspense and the page stayed "loading".
                {move || Suspend::new(async move { match user.await {
                    None => view! {
                        // guest
                        <A href="/login">"登录"</A>
                    }.into_any(),
                    Some(u) => view! {
                        // logged in — admins also see the publishing page link
                        {if u.role == "admin" {
                            view! { <A href="/admin">"管理后台"</A> }.into_any()
                        } else {
                            ().into_any()
                        }}
                        <A href="/profile" attr:class="username">{u.username}</A>
                        // ActionForm = an HTML <form> that calls a server
                        // function when submitted. Our first form!
                        <ActionForm action=logout>
                            <button type="submit">"退出登录"</button>
                        </ActionForm>
                    }.into_any(),
                }})}
            </Suspense>
        </nav>
    }
}
