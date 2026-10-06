use crate::auth::{Login, Signup, User};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// /login — two forms (log in / sign up) with a toggle between them.
#[component]
pub fn LoginPage() -> impl IntoView {
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
