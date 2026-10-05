// Root Leptos UI: document shell, shared user state, navigation, and route table.

use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes, A},
    hooks::{use_navigate},
    ParamSegment, StaticSegment,
};
use crate::auth::{get_user, Login, Logout, Signup, User};
// use crate::book::{get_books, get_chapter};
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
                        <A href="/profile" attr:class="username">{u.username}</A>
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
    let signup_role = RwSignal::new("student".to_string());

    // After a successful login/signup: reload the user and open their profile.
    let navigate = use_navigate();
    Effect::new(move |_| {
        let ok = matches!(login_action.value().get(), Some(Ok(())))
            || matches!(signup_action.value().get(), Some(Ok(())));
        if ok {
            user.refetch();
            navigate("/profile", Default::default());
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
                        <label>"I am signing up as"
                            <select
                                name="role"
                                required
                                prop:value=move || signup_role.get()
                                on:change=move |ev| signup_role.set(event_target_value(&ev))
                            >
                                <option value="student">"Student"</option>
                                <option value="parent">"Parent or guardian"</option>
                            </select>
                        </label>
                        <p class="auth-help">"For a student account, enter your details. For a parent account, enter the first child's details; use a link code if the child already has a student account."</p>
                        <label>"Learner name"
                            <input type="text" name="learner_name" maxlength="80"/>
                        </label>
                        <label>"Learner age range"
                            <select name="age_range">
                                <option value="under_8">"Under 8"</option>
                                <option value="8_10">"8–10"</option>
                                <option value="11_13" selected>"11–13"</option>
                                <option value="14_17">"14–17"</option>
                                <option value="18_plus">"18+"</option>
                            </select>
                        </label>
                        {move || if signup_role.get() == "parent" {
                            view! {
                                <div class="parent-child-setup">
                                    <p class="auth-help">"Already registered? Ask the student to generate a link code in their profile. Otherwise, create their login below."</p>
                                    <label>"Existing student link code (optional)"
                                        <input type="text" name="link_token" autocomplete="off"/>
                                    </label>
                                    <p class="auth-help">"Leave the code blank to create a new student login."</p>
                                    <label>"New student username"
                                        <input type="text" name="child_username" maxlength="64"/>
                                    </label>
                                    <label>"New student password (6+ characters)"
                                        <input type="password" name="child_password"/>
                                    </label>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <input type="hidden" name="link_token" value=""/>
                                <input type="hidden" name="child_username" value=""/>
                                <input type="hidden" name="child_password" value=""/>
                            }.into_any()
                        }}
                        <label>"Gender (optional)"
                            <select name="gender">
                                <option value="">"Prefer not to say"</option>
                                <option value="girl">"Girl / woman"</option>
                                <option value="boy">"Boy / man"</option>
                                <option value="another">"Another identity"</option>
                            </select>
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

