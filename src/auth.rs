// User identity, signup/login/logout server functions, and server-side session lookup.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// The logged-in user, as the browser is allowed to see it.
/// NOTE: no password_hash here — never let secrets cross the wire.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct User {
    pub id: i64,
    pub username: String,
    pub role: String, // "student", "parent", or "admin"
}

/// A vague "our fault" error for unexpected internal failures
/// (don't leak database/bcrypt details to the browser).
#[cfg(feature = "ssr")]
fn internal_error() -> ServerFnError {
    ServerFnError::ServerError("Something went wrong on our side.".into())
}

// ----------------------------------------------------------
//  SERVER FUNCTIONS — the four things the browser can ask us
// ----------------------------------------------------------

#[server]
pub async fn signup(
    username: String,
    password: String,
    role: String,
    learner_name: String,
    age_range: String,
    gender: String,
    child_username: Option<String>,
    child_password: Option<String>,
    link_token: Option<String>,
) -> Result<(), ServerFnError> {
    use self::ssr::*;
    use sqlx::Connection;

    let username = username.trim().to_string();
    if username.len() < 3 {
        return Err(ServerFnError::ServerError("Username needs at least 3 characters.".into()));
    }
    if password.len() < 6 {
        return Err(ServerFnError::ServerError("Password needs at least 6 characters.".into()));
    }
    if !matches!(role.as_str(), "student" | "parent") {
        return Err(ServerFnError::ServerError("Choose student or parent.".into()));
    }
    // Hash the password — the ONLY thing we ever store.
    let hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST).map_err(|_| internal_error())?;

    let mut conn = crate::db::db().await?;
    let mut tx = conn.begin().await?;

    // The very first account becomes the admin (that's you, the teacher).
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&mut *tx)
        .await?;
    // Preserve the prototype's first-account admin bootstrap. All later
    // accounts can choose whether they are a student or a parent.
    let actual_role = if count == 0 { "admin" } else { role.as_str() };

    let account = sqlx::query("INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)")
        .bind(&username)
        .bind(&hash)
        .bind(actual_role)
        .execute(&mut *tx)
        .await;
    let account = match account {
        Ok(row) => row,
        Err(_) => return Err(ServerFnError::ServerError("That username is taken.".into())),
    };
    let account_id = account.last_insert_rowid();

    if actual_role == "student" {
        validate_learner_details(&learner_name, &age_range, &gender)?;
        sqlx::query(
            "INSERT INTO learner_profiles (owner_user_id, name, age_range, gender)
             VALUES (?, ?, ?, ?)",
        )
        .bind(account_id)
        .bind(learner_name.trim())
        .bind(&age_range)
        .bind(&gender)
        .execute(&mut *tx)
        .await?;
    } else if actual_role == "parent" {
        let link_token = link_token.unwrap_or_default();
        if !link_token.trim().is_empty() {
            crate::classes::learning::attach_student_profile(
                &mut *tx,
                account_id,
                link_token.trim(),
                None,
                learner_name.trim(),
                &age_range,
                &gender,
            )
            .await?;
        } else {
            validate_learner_details(&learner_name, &age_range, &gender)?;
            let child_username = child_username.unwrap_or_default().trim().to_string();
            let child_password = child_password.unwrap_or_default();
            if child_username.len() < 3 || child_username.len() > 64 {
                return Err(ServerFnError::ServerError("Enter a new child username (3–64 characters), or use a link code.".into()));
            }
            if child_password.len() < 6 {
                return Err(ServerFnError::ServerError("Enter a child password of at least 6 characters, or use a link code.".into()));
            }
            let child_hash = bcrypt::hash(&child_password, bcrypt::DEFAULT_COST).map_err(|_| internal_error())?;
            let child = sqlx::query("INSERT INTO users (username, password_hash, role) VALUES (?, ?, 'student')")
                .bind(&child_username)
                .bind(&child_hash)
                .execute(&mut *tx)
                .await;
            let child = match child {
                Ok(row) => row,
                Err(_) => return Err(ServerFnError::ServerError("That child username is already taken.".into())),
            };
            sqlx::query(
                "INSERT INTO learner_profiles (owner_user_id, linked_user_id, name, age_range, gender)
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(account_id)
            .bind(child.last_insert_rowid())
            .bind(learner_name.trim())
            .bind(&age_range)
            .bind(&gender)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;
    tracing::info!(event = "signup", user = %username, role = %actual_role, "new user registered");
    create_session(account_id).await
}

#[cfg(feature = "ssr")]
fn validate_learner_details(name: &str, age_range: &str, gender: &str) -> Result<(), ServerFnError> {
    if name.trim().is_empty() || name.chars().count() > 80 {
        return Err(ServerFnError::ServerError("Enter a learner name (up to 80 characters).".into()));
    }
    if !matches!(age_range, "under_8" | "8_10" | "11_13" | "14_17" | "18_plus") {
        return Err(ServerFnError::ServerError("Choose an age range.".into()));
    }
    if !matches!(gender, "" | "girl" | "boy" | "another" | "prefer_not_to_say") {
        return Err(ServerFnError::ServerError("Choose a valid gender option.".into()));
    }
    Ok(())
}

#[server]
pub async fn login(username: String, password: String) -> Result<(), ServerFnError> {
    use self::ssr::*;

    let uname = username.trim().to_string();
    let mut conn = crate::db::db().await?;
    let row: Option<(i64, String)> =
        sqlx::query_as("SELECT id, password_hash FROM users WHERE username = ?")
            .bind(&uname)
            .fetch_optional(&mut conn)
            .await?;

    let Some((id, hash)) = row else {
        // Same message for "no such user" and "wrong password" —
        // don't tell attackers WHICH one they got right.
        tracing::warn!(event = "login_fail", user = %uname, "no such user");
        return Err(ServerFnError::ServerError("Wrong username or password.".into()));
    };

    let ok = bcrypt::verify(&password, &hash).map_err(|_| internal_error())?;
    if !ok {
        tracing::warn!(event = "login_fail", user = %uname, "wrong password");
        return Err(ServerFnError::ServerError("Wrong username or password.".into()));
    }

    tracing::info!(event = "login", user = %uname, user_id = id, "user logged in");
    create_session(id).await
}

#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    use self::ssr::*;
    destroy_session().await
}

/// "Who am I?" — pages and nav call this. Ok(None) = not logged in.
#[server]
pub async fn get_user() -> Result<Option<User>, ServerFnError> {
    use self::ssr::*;
    current_user().await
}

// ----------------------------------------------------------
//  SERVER-ONLY helpers
// ----------------------------------------------------------

#[cfg(feature = "ssr")]
pub mod ssr {
    use super::User;
    use crate::db::db;
    use http::header::{COOKIE, SET_COOKIE};
    use http::{request::Parts, HeaderValue};
    use leptos::prelude::*;
    use leptos_axum::ResponseOptions;

    /// Read the session cookie -> find the user it belongs to.
    /// EVERY protected server function will start with this line.
    pub async fn current_user() -> Result<Option<User>, ServerFnError> {
        let Some(token) = session_token() else {
            return Ok(None);
        };

        let mut conn = db().await?;
        let user = sqlx::query_as::<_, User>(
            "SELECT u.id, u.username, u.role
             FROM users u
             JOIN sessions s ON s.user_id = u.id
             WHERE s.token = ?",
        )
        .bind(token)
        .fetch_optional(&mut conn)
        .await?;

        Ok(user)
    }

    /// Require a valid session for protected server functions.
    pub async fn require_user() -> Result<User, ServerFnError> {
        current_user()
            .await?
            .ok_or_else(|| ServerFnError::ServerError("Please log in.".into()))
    }

    /// Pull our session token out of the request's Cookie header.
    fn session_token() -> Option<String> {
        let parts = use_context::<Parts>()?;
        let header = parts.headers.get(COOKIE)?.to_str().ok()?;
        header
            .split(';')
            .map(str::trim)
            .find_map(|c| c.strip_prefix("session="))
            .map(|s| s.to_string())
    }

    /// Log a user in: new session row + cookie with the token.
    /// The token is 32 random bytes from SQLite itself — no extra crates.
    pub async fn create_session(user_id: i64) -> Result<(), ServerFnError> {
        let mut conn = db().await?;

        let token: String = sqlx::query_scalar("SELECT hex(randomblob(32))")
            .fetch_one(&mut conn)
            .await?;

        sqlx::query("INSERT INTO sessions (token, user_id) VALUES (?, ?)")
            .bind(&token)
            .bind(user_id)
            .execute(&mut conn)
            .await?;

        // HttpOnly = JavaScript can NOT read this cookie (XSS protection).
        set_cookie(&format!(
            "session={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000"
        ));
        Ok(())
    }

    /// Log out: delete the session row + expire the cookie.
    pub async fn destroy_session() -> Result<(), ServerFnError> {
        if let Some(token) = session_token() {
            let mut conn = db().await?;
            sqlx::query("DELETE FROM sessions WHERE token = ?")
                .bind(token)
                .execute(&mut conn)
                .await?;
        }
        set_cookie("session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0");
        Ok(())
    }

    fn set_cookie(value: &str) {
        let resp = expect_context::<ResponseOptions>();
        if let Ok(v) = HeaderValue::from_str(value) {
            resp.insert_header(SET_COOKIE, v);
        }
    }
}
