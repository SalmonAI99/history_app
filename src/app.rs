// ============================================================
//  app.rs — your application.
//  Stage 6: the app now knows WHO you are.
//    /login  -> sign up / log in
//    header  -> "Log in" link, or your username + "Log out"
// ============================================================

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes, A},
    hooks::{use_navigate, use_params_map},
    ParamSegment, StaticSegment,
};
use crate::auth::{get_user, Login, Logout, Signup, User};
// use crate::book::{get_books, get_chapter};
use crate::classes;
use crate::files;
use crate::search;
use crate::site;

/// `shell` is the HTML "frame" around your app.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
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
            <header>
                <h1><A href="/">"阅读之家"</A></h1>
                // Site navigation (the public pages from site.rs):
                <nav class="site-nav">
                    <A href="/classes">"课程"</A>
                    <A href="/teacher">"老师"</A>
                    <A href="/blog">"文章"</A>
                    <A href="/news">"活动"</A>
                </nav>
                <form class="global-search" action="/search" method="get" role="search">
                    <label class="visually-hidden" for="global-search-input">"Search all content"</label>
                    <input id="global-search-input" type="search" name="q" placeholder="Search books, lessons, articles..."/>
                    <button type="submit">"Search"</button>
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
                    <Route path=StaticSegment("classes") view=classes::ClassesPage/>
                    <Route path=(StaticSegment("classes"), ParamSegment("id")) view=classes::ClassDetailPage/>
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
                {move || match user.get() {
                    None => ().into_any(), // still loading
                    Some(None) => view! {
                        // guest
                        <A href="/login">"Log in"</A>
                    }.into_any(),
                    Some(Some(u)) => view! {
                        // logged in — admins also see the publishing page link
                        {if u.role == "admin" {
                            view! { <A href="/admin">"Admin"</A> }.into_any()
                        } else {
                            ().into_any()
                        }}
                        <span class="username">{u.username}</span>
                        // ActionForm = an HTML <form> that calls a server
                        // function when submitted. Our first form!
                        <ActionForm action=logout>
                            <button type="submit">"Log out"</button>
                        </ActionForm>
                    }.into_any(),
                }}
            </Suspense>
        </nav>
    }
}

/// /login — two forms (log in / sign up) with a toggle between them.
#[component]
fn LoginPage() -> impl IntoView {
    let user = expect_context::<Resource<Option<User>>>();
    let login_action = ServerAction::<Login>::new();
    let signup_action = ServerAction::<Signup>::new();
    let show_signup = RwSignal::new(false);

    // After a successful login/signup: reload the user, go to the library.
    let navigate = use_navigate();
    Effect::new(move |_| {
        let ok = matches!(login_action.value().get(), Some(Ok(())))
            || matches!(signup_action.value().get(), Some(Ok(())));
        if ok {
            user.refetch();
            navigate("/", Default::default());
        }
    });

    view! {
        <div class="auth-form">
            {move || if show_signup.get() {
                view! {
                    <h2>"Create your account"</h2>
                    <ActionForm action=signup_action>
                        // "name" MUST match the server function's argument names
                        // (that's how the values find their way into the call).
                        <label>"Username"
                            <input type="text" name="username" required/>
                        </label>
                        <label>"Password (6+ characters)"
                            <input type="password" name="password" required/>
                        </label>
                        <button type="submit">"Sign up"</button>
                    </ActionForm>
                    // If the server function returned an error, show it:
                    {move || match signup_action.value().get() {
                        // peel off the "error running server function:" wrapper
                        Some(Err(ServerFnError::ServerError(msg))) => {
                            view! { <p class="error">{msg}</p> }.into_any()
                        }
                        _ => ().into_any(),
                    }}
                    <p class="auth-switch">
                        "Already have an account? "
                        <button on:click=move |_| show_signup.set(false)>"Log in"</button>
                    </p>
                }
                .into_any()
            } else {
                view! {
                    <h2>"欢迎回来"</h2>
                    <ActionForm action=login_action>
                        <label>"Username"
                            <input type="text" name="username" required/>
                        </label>
                        <label>"Password"
                            <input type="password" name="password" required/>
                        </label>
                        <button type="submit">"Log in"</button>
                    </ActionForm>
                    {move || match login_action.value().get() {
                        Some(Err(ServerFnError::ServerError(msg))) => {
                            view! { <p class="error">{msg}</p> }.into_any()
                        }
                        _ => ().into_any(),
                    }}
                    <p class="auth-switch">
                        "New here? "
                        <button on:click=move |_| show_signup.set(true)>"Create an account"</button>
                    </p>
                }
                .into_any()
            }}
        </div>
    }
}

// ----------------------------------------------------------
//  PAGE: THE LIBRARY (moved from "/" to "/library" in Stage 8)
// ----------------------------------------------------------

// #[component]
// fn LibraryPage() -> impl IntoView {
//     let books = Resource::new(|| (), |_| get_books());
//
//     view! {
//         <h2>"Library"</h2>
//         <Transition fallback=move || view! { <p class="loading">"Loading..."</p> }>
//             {move || match books.get() {
//                 None => ().into_any(),
//                 Some(Err(_)) => view! {
//                     <p>"Could not load the library."</p>
//                 }.into_any(),
//                 Some(Ok(books)) => view! {
//                     <ul class="book-list">
//                         {books.into_iter().map(|book| view! {
//                             <li>
//                                 <div class="book-row">
//                                     <strong>{book.title.clone()}</strong>
//                                     <A attr:class="button" href=format!("/book/{}/chapter/1", book.id)>
//                                         "Open book →"
//                                     </A>
//                                 </div>
//                             </li>
//                         }).collect_view()}
//                     </ul>
//                 }.into_any(),
//             }}
//         </Transition>
//     }
// }
//
// // ----------------------------------------------------------
// //  PAGE: THE READER (unchanged from Stage 4)
// // ----------------------------------------------------------
//
// #[component]
// fn ReaderPage() -> impl IntoView {
//     let params = use_params_map();
//
//     let ids = move || {
//         let p = params.read();
//         let book_id = p.get("id").and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
//         let chapter = p.get("chapter").and_then(|s| s.parse::<i64>().ok()).unwrap_or(1);
//         (book_id, chapter)
//     };
//
//     let chapter = Resource::new(
//         move || ids(),
//         move |(book_id, ch)| get_chapter(book_id, ch),
//     );
//
//     view! {
//         <div class="reader">
//             <Transition fallback=move || view! { <p class="loading">"Loading..."</p> }>
//                 {move || match chapter.get() {
//                     None => ().into_any(),
//
//                     Some(Err(_)) => view! {
//                         <p>"Sorry, this chapter could not be found."</p>
//                         <p><A href="/library">"← Back to the library"</A></p>
//                     }.into_any(),
//
//                     Some(Ok(page)) => {
//                         let (book_id, ch) = ids();
//                         let total = page.total_chapters;
//
//                         view! {
//                             <h2 id="chapter-title">{page.title}</h2>
//
//                             <article id="book-content">
//                                 {page.paragraphs
//                                     .iter()
//                                     .map(|p| view! { <p>{p.clone()}</p> })
//                                     .collect_view()}
//                             </article>
//
//                             <nav class="navigation">
//                                 {if ch > 1 {
//                                     view! {
//                                         <A attr:class="button" href=format!("/book/{book_id}/chapter/{}", ch - 1)>
//                                             "← Previous"
//                                         </A>
//                                     }.into_any()
//                                 } else {
//                                     view! { <span class="button disabled">"← Previous"</span> }.into_any()
//                                 }}
//
//                                 <span id="page-number">{format!("{ch} / {total}")}</span>
//
//                                 {if ch < total {
//                                     view! {
//                                         <A attr:class="button" href=format!("/book/{book_id}/chapter/{}", ch + 1)>
//                                             "Next →"
//                                         </A>
//                                     }.into_any()
//                                 } else {
//                                     view! { <span class="button disabled">"Next →"</span> }.into_any()
//                                 }}
//
//                                 <A attr:class="button" href=format!("/book/{book_id}/chapter/1")>"Reset"</A>
//                             </nav>
//
//                         }.into_any()
//                     }
//                 }}
//             </Transition>
//         </div>
//     }
// }
