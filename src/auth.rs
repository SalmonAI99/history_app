// ============================================================
//  auth.rs — who are you?
//  The three ideas of this stage:
//   1. PASSWORD HASHING: we never store a real password, only
//      its bcrypt hash (a ONE-WAY fingerprint). Even if someone
//      steals our database, they can't log in with the hashes.
//   2. SESSIONS: after login, the server hands the browser a
//      random token (a "session"). The token means nothing by
//      itself — it's a key to a row in our sessions table.
//   3. COOKIES: the browser stores the token and sends it with
//      EVERY request automatically. That's how the server knows
//      "oh, it's Emma again" without a new login every click.
// ============================================================

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// The logged-in user, as the browser is allowed to see it.
/// NOTE: no password_hash here — never let secrets cross the wire.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct User {
    pub id: i64,
    pub username: String,
    pub role: String, // "student" or "admin"
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
pub async fn signup(username: String, password: String) -> Result<(), ServerFnError> {
    use self::ssr::*;

    let username = username.trim().to_string();
    if username.len() < 3 {
        return Err(ServerFnError::ServerError("Username needs at least 3 characters.".into()));
    }
    if password.len() < 6 {
        return Err(ServerFnError::ServerError("Password needs at least 6 characters.".into()));
    }

    // Hash the password — the ONLY thing we ever store.
    let hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST).map_err(|_| internal_error())?;

    let mut conn = crate::db::db().await?;

    // The very first account becomes the admin (that's you, the teacher).
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&mut conn)
        .await?;
    let role = if count == 0 { "admin" } else { "student" };

    let result = sqlx::query("INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)")
        .bind(&username)
        .bind(&hash)
        .bind(&role)
        .execute(&mut conn)
        .await;

    match result {
        Ok(r) => {
            // TELEMETRY: a line you'll grep when counting signups
            tracing::info!(event = "signup", user = %username, role = %role, "new user registered");
            create_session(r.last_insert_rowid()).await
        }
        Err(_) => {
            tracing::warn!(event = "signup_fail", user = %username, "username already taken");
            Err(ServerFnError::ServerError("That username is taken.".into()))
        }
    }
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
