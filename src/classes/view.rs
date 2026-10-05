//! Leptos page components for the learner profile, course catalog, and course detail routes.

use leptos::prelude::*;
use crate::auth::User;
use crate::classes::learning::{get_learning_dashboard, LearningDashboard};
use leptos_router::{
    components::A,
    hooks::{use_params_map, use_query_map},
};
use crate::classes::slides::{get_unlocked_courses, SlideDeckViewer, ALL_DECKS};
use crate::classes::videos::{collect_course_videos, CourseVideoLibrary};
use crate::classes::{ALL_COURSES, get_class_comments, CourseMindMap, course_thumbnail, AddClassComment};
use leptos_meta::Title;
#[component]
pub fn ProfilePage() -> impl IntoView {
    let user = expect_context::<Resource<Option<User>>>();
    let query_map = use_query_map();
    let dashboard = Resource::new(|| (), |_| get_learning_dashboard());
    let selected_learner = RwSignal::new(
        query_map
            .read_untracked()
            .get("learner")
            .and_then(|id| id.parse::<i64>().ok()),
    );
    let learner_choices = RwSignal::new(Vec::<(i64, String, bool)>::new());
    let link_target_id = RwSignal::new(0_i64);
    let add_learner = ServerAction::<crate::classes::learning::CreateChildAccount>::new();
    let link_child = ServerAction::<crate::classes::learning::LinkExistingChild>::new();
    let make_link_token = ServerAction::<crate::classes::learning::GenerateParentLinkToken>::new();
    let assign_course = ServerAction::<crate::classes::learning::AssignCourseToLearner>::new();
    let observed_action_versions = RwSignal::new((0_usize, 0_usize, 0_usize));

    Effect::new(move |_| {
        if let Some(Ok(data)) = dashboard.get() {
            learner_choices.set(data.learners.iter().map(|entry| {
                (entry.learner.id, entry.learner.name.clone(), entry.learner.account_username.is_some())
            }).collect());
            let requested_id = query_map.read().get("learner").and_then(|id| id.parse::<i64>().ok());
            let requested_is_valid = requested_id.is_some_and(|id| data.learners.iter().any(|entry| entry.learner.id == id));
            let current_id = selected_learner.get_untracked();
            let selected_is_valid = current_id.is_some_and(|id| data.learners.iter().any(|entry| entry.learner.id == id));
            let desired_id = if requested_is_valid {
                requested_id
            } else if selected_is_valid {
                current_id
            } else {
                data.learners.first().map(|entry| entry.learner.id)
            };
            if current_id != desired_id {
                selected_learner.set(desired_id);
            }
        }
    });
    Effect::new(move |_| {
        let learner_id = selected_learner.get();
        let target_id = learner_choices.get().into_iter()
            .find(|(id, _, is_linked)| Some(*id) == learner_id && !is_linked)
            .map(|(id, _, _)| id)
            .unwrap_or_default();
        if link_target_id.get_untracked() != target_id {
            link_target_id.set(target_id);
        }
    });
    // Keep the existing dashboard visible during refetches and refresh it only after a successful mutation.
    Effect::new(move |_| {
        let versions = (
            add_learner.version().get(),
            link_child.version().get(),
            assign_course.version().get(),
        );
        let previous = observed_action_versions.get_untracked();
        if versions == previous {
            return;
        }

        let succeeded = (versions.0 != previous.0 && matches!(add_learner.value().get(), Some(Ok(()))))
            || (versions.1 != previous.1 && matches!(link_child.value().get(), Some(Ok(()))))
            || (versions.2 != previous.2 && matches!(assign_course.value().get(), Some(Ok(()))));
        observed_action_versions.set(versions);
        if succeeded {
            dashboard.refetch();
        }
    });

    view! {
        <Title text="Profile"/>
        <section class="classes-page profile-page">
            <div class="classes-heading">
                <p class="classes-eyebrow">"YOUR ACCOUNT"</p>
                <h2>"Profile"</h2>
                <p>"Manage learner accounts, course assignments, and learning progress."</p>
            </div>
            <Suspense fallback=|| ().into_view()>
                {move || match user.get() {
                    None => ().into_any(),
                    Some(None) => view! { <p class="classes-login-prompt">"Please " <A href="/login">"log in"</A> " to see your profile."</p> }.into_any(),
                    Some(Some(account)) => {
                        let is_parent = account.role == "parent";
                        let is_student = account.role == "student";
                        let is_admin = account.role == "admin";
                        view! {
                            {if !is_admin {
                                view! {
                                    <LearnerLearningPanel
                                        dashboard=dashboard
                                        selected_learner=selected_learner
                                        learner_choices=learner_choices
                                        is_parent=is_parent
                                        is_student=is_student
                                        make_link_token=make_link_token
                                        add_learner=add_learner
                                        link_child=link_child
                                        link_target_id=link_target_id
                                    />
                                }.into_any()
                            } else { ().into_any() }}

                            {if is_admin {
                                view! {
                                    <section class="learning-dashboard">
                                        <h2>"Admin account"</h2>
                                        <p>"Learner profiles and course progress are available to student and parent accounts."</p>
                                        <A href="/admin" attr:class="class-card-action">"Open admin tools →"</A>
                                    </section>
                                }.into_any()
                            } else { ().into_any() }}

                            {if is_parent {
                                view! {
                                    <PurchasedCoursesPanel
                                        dashboard=dashboard
                                        selected_learner=selected_learner
                                        assign_course=assign_course
                                    />
                                }.into_any()
                            } else { ().into_any() }}

                            {if !is_admin {
                                view! {
                                    <div class="profile-catalog-link">
                                        <p>"Ready to find another course?"</p>
                                        <A href=move || {
                                            selected_learner.get()
                                                .map(|id| format!("/classes?learner={id}"))
                                                .unwrap_or_else(|| "/classes".to_string())
                                        } attr:class="class-card-action">"Browse course catalog →"</A>
                                    </div>
                                }.into_any()
                            } else { ().into_any() }}
                        }.into_any()
                    }
                }}
            </Suspense>
        </section>
    }
}

#[component]
fn LearnerLearningPanel(
    dashboard: Resource<Result<LearningDashboard, ServerFnError>>,
    selected_learner: RwSignal<Option<i64>>,
    learner_choices: RwSignal<Vec<(i64, String, bool)>>,
    is_parent: bool,
    is_student: bool,
    make_link_token: ServerAction<crate::classes::learning::GenerateParentLinkToken>,
    add_learner: ServerAction<crate::classes::learning::CreateChildAccount>,
    link_child: ServerAction<crate::classes::learning::LinkExistingChild>,
    link_target_id: RwSignal<i64>,
) -> impl IntoView {
    view! {
        <section class="learning-dashboard" aria-label="Learning dashboard">
            <div class="dashboard-heading">
                <div>
                    <p class="classes-eyebrow">"YOUR LEARNING"</p>
                    <h2>{if is_parent { "Learner dashboard" } else { "My courses" }}</h2>
                </div>
                {if is_parent {
                    view! {
                        <LearnerPicker selected_learner=selected_learner learner_choices=learner_choices/>
                    }.into_any()
                } else { ().into_any() }}
            </div>

            <LearnerCourseOverview dashboard=dashboard selected_learner=selected_learner is_parent=is_parent/>

            {if is_student {
                view! { <StudentAccountLinkPanel dashboard=dashboard make_link_token=make_link_token/> }.into_any()
            } else { ().into_any() }}

            {if is_parent {
                view! {
                    <ParentLearnerManagement
                        add_learner=add_learner
                        link_child=link_child
                        link_target_id=link_target_id
                    />
                }.into_any()
            } else { ().into_any() }}
        </section>
    }
}

#[component]
fn LearnerPicker(
    selected_learner: RwSignal<Option<i64>>,
    learner_choices: RwSignal<Vec<(i64, String, bool)>>,
) -> impl IntoView {
    view! {
        <label class="learner-picker">
            <span>"Viewing learner"</span>
            <select
                on:change=move |ev| selected_learner.set(event_target_value(&ev).parse::<i64>().ok())
                prop:value=move || selected_learner.get().unwrap_or_default().to_string()
            >
                {move || learner_choices.get().into_iter().map(|(id, name, _)| {
                    view! { <option value=id>{name}</option> }
                }).collect_view()}
            </select>
        </label>
    }
}

#[component]
fn LearnerCourseOverview(
    dashboard: Resource<Result<LearningDashboard, ServerFnError>>,
    selected_learner: RwSignal<Option<i64>>,
    is_parent: bool,
) -> impl IntoView {
    view! {
        <Transition fallback=move || view! { <p class="dashboard-loading">"Loading learner profiles…"</p> }>
            {move || match dashboard.get() {
                None => ().into_any(),
                Some(Err(_)) => view! { <p class="dashboard-error">"Could not load the learning dashboard."</p> }.into_any(),
                Some(Ok(data)) => {
                    let selected_id = selected_learner.get().filter(|id| data.learners.iter().any(|l| l.learner.id == *id))
                        .or_else(|| data.learners.first().map(|l| l.learner.id));
                    if let Some(profile) = data.learners.iter().find(|l| Some(l.learner.id) == selected_id) {
                        let learner_name = profile.learner.name.clone();
                        view! {
                            <div class="learner-dashboard-content">
                                <div class="learner-profile-summary">
                                    <div>
                                        <h3>{learner_name}</h3>
                                        <p>{format!("Age range: {} · Gender: {}", age_range_label(&profile.learner.age_range), gender_label(&profile.learner.gender))}</p>
                                        {if is_parent {
                                            profile.learner.account_username.as_ref().map(|username| view! {
                                                <p>{format!("Student login: {username}")}</p>
                                            }.into_any()).unwrap_or_else(|| view! { <p>"No student login is linked yet."</p> }.into_any())
                                        } else {
                                            view! { <p>"Student account"</p> }.into_any()
                                        }}
                                    </div>
                                    <span class="learner-course-count">{format!("{} courses", profile.courses.len())}</span>
                                </div>
                                {if profile.courses.is_empty() {
                                    view! { <p class="empty-learning-state">"No courses assigned yet. Browse the catalog to get started."</p> }.into_any()
                                } else {
                                    view! {
                                        <div class="learner-course-list">
                                            {profile.courses.iter().map(|course| {
                                                let course_id = course.course_id;
                                                let learner_id = profile.learner.id;
                                                let href = course.current_deck.as_ref().map(|deck| {
                                                    format!("/classes/{course_id}?deck={deck}&learner={learner_id}#slide-{}", course.current_slide)
                                                }).unwrap_or_else(|| format!("/classes/{course_id}?learner={learner_id}"));
                                                let title = course.title.clone();
                                                let progress = course.progress_percent;
                                                let progress_text = if let Some(deck) = &course.current_deck {
                                                    format!("Last opened: {deck}, slide {}", course.current_slide)
                                                } else { "Not started yet".to_string() };
                                                view! {
                                                    <article class="learner-course-card">
                                                        <div class="learner-course-card-top">
                                                            <div><h4>{title}</h4><p>{progress_text}</p></div>
                                                            <strong>{format!("{progress}%")}</strong>
                                                        </div>
                                                        <div class="course-progress-track" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow=progress>
                                                            <span style:width=format!("{progress}%")></span>
                                                        </div>
                                                        <A href=href attr:class="dashboard-course-link">"Continue course →"</A>
                                                    </article>
                                                }
                                            }).collect_view()}
                                        </div>
                                    }.into_any()
                                }}
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <div class="empty-learning-state">
                                <p>"Add a learner profile to start assigning courses."</p>
                            </div>
                        }.into_any()
                    }
                }
            }}
        </Transition>
    }
}

#[component]
fn StudentAccountLinkPanel(
    dashboard: Resource<Result<LearningDashboard, ServerFnError>>,
    make_link_token: ServerAction<crate::classes::learning::GenerateParentLinkToken>,
) -> impl IntoView {
    view! {
        <Transition fallback=|| ().into_view()>
            {move || {
                let linked = dashboard.get().and_then(Result::ok)
                    .is_some_and(|data| data.learners.iter().any(|entry| entry.learner.account_username.is_some()));
                if linked {
                    view! { <p class="account-linked-note">"This student account is linked to a parent."</p> }.into_any()
                } else {
                    view! {
                        <details class="account-link-panel">
                            <summary>"Link this account to a parent"</summary>
                            <p>"Generate a one-time code and share it with your parent. It expires in 30 minutes."</p>
                            <ActionForm action=make_link_token>
                                <button type="submit">"Generate parent link code"</button>
                            </ActionForm>
                            {move || match make_link_token.value().get() {
                                Some(Ok(token)) => view! { <p class="link-token-display">{token}</p> }.into_any(),
                                Some(Err(ServerFnError::ServerError(msg))) => view! { <p class="error">{msg}</p> }.into_any(),
                                _ => ().into_any(),
                            }}
                        </details>
                    }.into_any()
                }
            }}
        </Transition>
    }
}

#[component]
fn ParentLearnerManagement(
    add_learner: ServerAction<crate::classes::learning::CreateChildAccount>,
    link_child: ServerAction<crate::classes::learning::LinkExistingChild>,
    link_target_id: RwSignal<i64>,
) -> impl IntoView {
    let show_existing_account_form = RwSignal::new(false);
    let show_create_account_dialog = RwSignal::new(false);

    view! {
        <section class="add-learner-panel" aria-labelledby="connect-student-heading">
            <h3 id="connect-student-heading">"Connect a student account"</h3>
            <p class="auth-help">"Choose whether your child already has a student account."</p>
            <div class="learner-link-options">
                <button
                    type="button"
                    class="learner-link-option"
                    aria-expanded=move || show_existing_account_form.get().to_string()
                    on:click=move |_| show_existing_account_form.set(true)
                >
                    <strong>"My child already has an account"</strong>
                    <span>"Link it with the one-time code from their profile."</span>
                </button>
                <button
                    type="button"
                    class="learner-link-option"
                    on:click=move |_| show_create_account_dialog.set(true)
                >
                    <strong>"My child needs an account"</strong>
                    <span>"Set up their student profile and login."</span>
                </button>
            </div>

            {move || if show_existing_account_form.get() {
                view! {
                    <div class="existing-child-link">
                        <div class="existing-child-link-heading">
                            <div>
                                <h4>"Link an existing account"</h4>
                                <p class="auth-help">"Ask your child to generate a code from their profile. It expires in 30 minutes."</p>
                            </div>
                            <button
                                type="button"
                                class="text-button"
                                on:click=move |_| show_existing_account_form.set(false)
                            >"Cancel"</button>
                        </div>
                        <ActionForm action=link_child>
                            <input type="hidden" name="learner_id" prop:value=move || link_target_id.get().to_string()/>
                            <label>"One-time code"<input name="token" autocomplete="off" required/></label>
                            <button type="submit">"Link student account"</button>
                        </ActionForm>
                        {move || match link_child.value().get() {
                            Some(Ok(())) => view! { <p class="ok">"Student account linked successfully."</p> }.into_any(),
                            Some(Err(ServerFnError::ServerError(msg))) => view! { <p class="error">{msg}</p> }.into_any(),
                            Some(Err(error)) => view! { <p class="error">{error.to_string()}</p> }.into_any(),
                            _ => ().into_any(),
                        }}
                    </div>
                }.into_any()
            } else { ().into_any() }}

            {move || if show_create_account_dialog.get() {
                view! {
                    <div
                        class="learner-setup-backdrop"
                        role="presentation"
                        on:click=move |_| show_create_account_dialog.set(false)
                    >
                        <section
                            class="learner-setup-dialog"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="learner-setup-title"
                            on:click=move |ev| ev.stop_propagation()
                        >
                            <div class="learner-setup-heading">
                                <div>
                                    <p class="classes-eyebrow">"NEW STUDENT"</p>
                                    <h3 id="learner-setup-title">"Set up your child's account"</h3>
                                    <p class="auth-help">"Create a learner profile and student login. You can use the login to access their courses."</p>
                                </div>
                                <button
                                    type="button"
                                    class="modal-close-button"
                                    aria-label="Close account setup"
                                    on:click=move |_| show_create_account_dialog.set(false)
                                >"×"</button>
                            </div>
                            <ActionForm action=add_learner attr:class="learner-setup-form">
                                <label>"Child's name"<input name="name" maxlength="80" required autofocus/></label>
                                <label>"Age range"
                                    <select name="age_range" required>
                                        <option value="under_8">"Under 8"</option>
                                        <option value="8_10">"8–10"</option>
                                        <option value="11_13" selected>"11–13"</option>
                                        <option value="14_17">"14–17"</option>
                                        <option value="18_plus">"18+"</option>
                                    </select>
                                </label>
                                <label>"Gender (optional)"
                                    <select name="gender">
                                        <option value="">"Prefer not to say"</option>
                                        <option value="girl">"Girl / woman"</option>
                                        <option value="boy">"Boy / man"</option>
                                        <option value="another">"Another identity"</option>
                                    </select>
                                </label>
                                <label>"Student username"<input name="username" maxlength="64" required/></label>
                                <label>"Student password (6+ characters)"<input name="password" type="password" required/></label>
                                <div class="learner-setup-actions">
                                    <button type="button" class="secondary-button" on:click=move |_| show_create_account_dialog.set(false)>"Cancel"</button>
                                    <button type="submit">"Create student account"</button>
                                </div>
                            </ActionForm>
                            {move || match add_learner.value().get() {
                                Some(Ok(())) => view! { <p class="ok">"Student account created and linked successfully."</p> }.into_any(),
                                Some(Err(ServerFnError::ServerError(msg))) => view! { <p class="error">{msg}</p> }.into_any(),
                                Some(Err(error)) => view! { <p class="error">{error.to_string()}</p> }.into_any(),
                                _ => ().into_any(),
                            }}
                        </section>
                    </div>
                }.into_any()
            } else { ().into_any() }}
        </section>
    }
}

#[component]
fn PurchasedCoursesPanel(
    dashboard: Resource<Result<LearningDashboard, ServerFnError>>,
    selected_learner: RwSignal<Option<i64>>,
    assign_course: ServerAction<crate::classes::learning::AssignCourseToLearner>,
) -> impl IntoView {
    view! {
        <section class="purchased-courses-panel">
            <div class="catalog-heading">
                <div><p class="classes-eyebrow">"COURSE ACCESS"</p><h2>"Purchased courses"</h2></div>
                <p>"Assign a purchased course to the selected learner."</p>
            </div>
            <Transition fallback=|| view! { <p class="dashboard-loading">"Loading course access…"</p> }>
                {move || match dashboard.get() {
                    None => ().into_any(),
                    Some(Err(_)) => view! { <p class="dashboard-error">"Could not load purchased courses."</p> }.into_any(),
                    Some(Ok(data)) => {
                        let selected_id = selected_learner.get().filter(|id| data.learners.iter().any(|entry| entry.learner.id == *id))
                            .or_else(|| data.learners.first().map(|entry| entry.learner.id));
                        if let Some(profile) = data.learners.iter().find(|entry| Some(entry.learner.id) == selected_id) {
                            let available: Vec<_> = data.purchased_course_ids.iter()
                                .filter_map(|id| ALL_COURSES.iter().find(|course| course.id == *id))
                                .filter(|course| !profile.courses.iter().any(|assigned| assigned.course_id == course.id))
                                .collect();
                            if available.is_empty() {
                                view! { <p class="empty-learning-state">"All purchased courses are assigned to this learner."</p> }.into_any()
                            } else {
                                let learner_id = profile.learner.id;
                                let assign_action = assign_course;
                                view! {
                                    <ul class="purchased-course-list">
                                        {available.into_iter().map(|course| {
                                            let course_id = course.id;
                                            view! {
                                                <li>
                                                    <span>{course.title}</span>
                                                    <ActionForm action=assign_action>
                                                        <input type="hidden" name="learner_id" value=learner_id/>
                                                        <input type="hidden" name="course_id" value=course_id/>
                                                        <button type="submit" class="assign-course-button">"Assign to learner"</button>
                                                    </ActionForm>
                                                </li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                }.into_any()
                            }
                        } else {
                            view! { <p class="empty-learning-state">"Add a learner profile before assigning courses."</p> }.into_any()
                        }
                    }
                }}
            </Transition>
            {move || match assign_course.value().get() {
                Some(Err(ServerFnError::ServerError(msg))) => view! { <p class="error">{msg}</p> }.into_any(),
                _ => ().into_any(),
            }}
        </section>
    }
}

#[component]
pub fn ClassesPage() -> impl IntoView {
    let user = expect_context::<Resource<Option<User>>>();
    let query_map = use_query_map();
    let learner_id = move || query_map.read().get("learner").and_then(|id| id.parse::<i64>().ok());

    view! {
        <Title text="Classes"/>
        <section class="classes-page">
            <div class="classes-heading">
                <p class="classes-eyebrow">"LEARN & EXPLORE"</p>
                <h2>"Classes"</h2>
                <p>"Explore guided courses with videos, interactive learning maps, and lesson decks."</p>
            </div>
            <Suspense fallback=|| ().into_view()>
                {move || match user.get() {
                    None => ().into_any(),
                    Some(None) => view! { <p class="classes-login-prompt">"Please " <A href="/login">"log in"</A> " to browse the course catalog."</p> }.into_any(),
                    Some(Some(_)) => view! {
                        <ul class="class-grid">
                            {move || ALL_COURSES.iter().map(|course| {
                                let course_id = course.id;
                                let thumbnail = course_thumbnail(course);
                                let title = course.title;
                                let description = if course.notes.is_empty() {
                                    "Explore the course overview, syllabus, and lessons.".to_string()
                                } else { course.notes.to_string() };
                                let course_meta = format!("{} weeks · {} lesson decks", course.weeks.len(), course.slide_decks.len());
                                let href = learner_id().map(|id| format!("/classes/{course_id}?learner={id}"))
                                    .unwrap_or_else(|| format!("/classes/{course_id}"));
                                view! {
                                    <li class="class-card">
                                        <div class="class-card-thumbnail">
                                            {match thumbnail {
                                                Some(src) => view! { <img src=src alt=format!("{title} course thumbnail") loading="lazy"/> }.into_any(),
                                                None => view! { <div class="class-card-placeholder" aria-hidden="true"><span>"COURSE"</span></div> }.into_any(),
                                            }}
                                            <span class="class-card-thumbnail-label">"INTERACTIVE COURSE"</span>
                                        </div>
                                        <div class="class-card-content">
                                            <h3>{title}</h3>
                                            <p>{description}</p>
                                            <div class="class-card-meta">{course_meta}</div>
                                            <div class="catalog-card-actions">
                                                <A href=href attr:class="class-card-action">"Explore class →"</A>
                                            </div>
                                        </div>
                                    </li>
                                }
                            }).collect_view()}
                        </ul>
                    }.into_any(),
                }}
            </Suspense>
        </section>
    }
}

fn age_range_label(value: &str) -> &str {
    match value {
        "under_8" => "under 8",
        "8_10" => "8–10",
        "11_13" => "11–13",
        "14_17" => "14–17",
        "18_plus" => "18+",
        _ => "not set",
    }
}

fn gender_label(value: &str) -> &str {
    match value {
        "girl" => "girl / woman",
        "boy" => "boy / man",
        "another" => "another identity",
        "prefer_not_to_say" | "" => "not specified",
        _ => "not specified",
    }
}

#[component]
pub fn ClassDetailPage() -> impl IntoView {
    let params = use_params_map();
    let query_map = use_query_map();
    let class_id = move || params.read().get("id").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    let user = expect_context::<Resource<Option<User>>>();
    let add_comment = ServerAction::<AddClassComment>::new();
    let comments = Resource::new(move || (class_id(), add_comment.version().get(), user.get()), move |(id, _, u)| async move { if let Some(Some(_)) = u { get_class_comments(id).await } else { Ok(vec![]) } });

    // Deck IDs this account may watch (admin, subscription, or an enrolled learner).
    let unlocked_decks = Resource::new(
        move || user.get().flatten().map(|u| u.id),
        |_| async { get_unlocked_courses().await.unwrap_or_default() },
    );

    let active_deck = move || query_map.read().get("deck");
    let active_learner_id = move || query_map.read().get("learner").and_then(|id| id.parse::<i64>().ok());

    view! {
<Suspense fallback=|| view! { <p>"Loading class…"</p> }>

            // Suspend + .await (not .get()): with .get() the server sometimes
            // never closed this Suspense and the page stuck on "loading".
            {move || Suspend::new(async move {
                let user = user.await;
                let unlocked_list = unlocked_decks.await;
                match user {
                None => view! { <p>"Please " <A href="/login">"log in"</A> " to view this class."</p> }.into_any(),
                Some(_) => match ALL_COURSES.iter().find(|c| c.id == class_id()) {
                    None => view! { <p>"Class not found."</p> <A href="/classes">"← All classes"</A> }.into_any(),
                    Some(class) => {
                        let course_videos = collect_course_videos(class.slide_decks);
                        let videos_unlocked = Signal::stored(
                            unlocked_list.iter().any(|id| class.slide_decks.contains(&id.as_str())),
                        );
                        let video_count = course_videos.len();
                        let slide_count: usize = ALL_DECKS
                            .iter()
                            .filter(|deck| class.slide_decks.contains(&deck.id))
                            .map(|deck| deck.slide_count)
                            .sum();

                        view! { <div class="course-root">
                        {move || match active_deck() {
                            Some(deck_id) => {
                                match ALL_DECKS.iter().find(|d| d.id == deck_id) {
                            Some(deck) => view! {
                                <div class="slide-viewer-wrapper">
                                    <div class="slide-viewer-nav">
                                                <A href=active_learner_id().map(|id| format!("/classes/{}?learner={id}", class.id)).unwrap_or_else(|| format!("/classes/{}", class.id)) attr:class="slide-viewer-back-btn">
                                                    "← Back to Course Overview"
                                                </A>
                                            </div>
                                            <SlideDeckViewer deck=*deck learner_id=active_learner_id().unwrap_or_default() />
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
                                            <CourseVideoLibrary videos=course_videos.clone() unlocked=videos_unlocked/>
                                        </section>

                                        <section id="course-roadmap" class="course-roadmap-section">
                                            <div class="course-section-heading">
                                                <div>
                                                    <p class="course-eyebrow">"YOUR LEARNING PATH"</p>
                                                    <h2 class="course-section-title">"Interactive syllabus"</h2>
                                                </div>
                                                <span class="course-section-count">{format!("{} weeks", class.weeks.len())}</span>
                                            </div>
                                            <CourseMindMap course_id=class.id course_title=class.title weeks=class.weeks/>
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
                                                                <A href=active_learner_id().map(|id| format!("/classes/{}?deck={}&learner={id}", class.id, d_id)).unwrap_or_else(|| format!("/classes/{}?deck={}", class.id, d_id)) attr:class="curriculum-button">
                                                                    "Open lesson deck →"
                                                                </A>
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
                                                    <input type="hidden" name="class_id" prop:value=move || class_id().to_string()/>
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
            }}})}
</Suspense>
    }
}
