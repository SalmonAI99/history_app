// ============================================================
//  slides/access.rs — Course Access & Purchase Permissions
// ============================================================

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourseAccessStatus {
    pub is_unlocked: bool,
    pub is_admin: bool,
    pub is_logged_in: bool,
    pub username: Option<String>,
}

impl Default for CourseAccessStatus {
    fn default() -> Self {
        Self {
            is_unlocked: false,
            is_admin: false,
            is_logged_in: false,
            username: None,
        }
    }
}

/// Checks whether the current user has access to view full notes for `deck_id`.
#[server]
pub async fn check_course_access(deck_id: String) -> Result<CourseAccessStatus, ServerFnError> {
    let maybe_user = crate::auth::ssr::current_user().await?;
    let Some(user) = maybe_user else {
        return Ok(CourseAccessStatus {
            is_unlocked: false,
            is_admin: false,
            is_logged_in: false,
            username: None,
        });
    };

    // Admins always have access to all course notes:
    if user.role == "admin" {
        return Ok(CourseAccessStatus {
            is_unlocked: true,
            is_admin: true,
            is_logged_in: true,
            username: Some(user.username),
        });
    }

    let mut conn = crate::db::db().await?;

    // Check specific course purchase / grant:
    let (has_course,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM course_access WHERE user_id = ? AND deck_id = ?",
    )
    .bind(user.id)
    .bind(&deck_id)
    .fetch_one(&mut conn)
    .await?;

    if has_course > 0 {
        return Ok(CourseAccessStatus {
            is_unlocked: true,
            is_admin: false,
            is_logged_in: true,
            username: Some(user.username),
        });
    }

    // Check subscription / member status (Stage 11):
    let has_sub: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT paid_until FROM subscriptions WHERE user_id = ? AND paid_until > datetime('now')",
    )
    .bind(user.id)
    .fetch_optional(&mut conn)
    .await?;

    let is_unlocked = has_sub.is_some();

    Ok(CourseAccessStatus {
        is_unlocked,
        is_admin: false,
        is_logged_in: true,
        username: Some(user.username),
    })
}

/// Returns a list of all deck IDs unlocked by the current user.
#[server]
pub async fn get_unlocked_courses() -> Result<Vec<String>, ServerFnError> {
    let maybe_user = crate::auth::ssr::current_user().await?;
    let Some(user) = maybe_user else {
        return Ok(Vec::new());
    };

    if user.role == "admin" {
        return Ok(crate::classes::slides::ALL_DECKS.iter().map(|d| d.id.to_string()).collect());
    }

    let mut conn = crate::db::db().await?;

    let has_sub: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT paid_until FROM subscriptions WHERE user_id = ? AND paid_until > datetime('now')",
    )
    .bind(user.id)
    .fetch_optional(&mut conn)
    .await?;

    if has_sub.is_some() {
        return Ok(crate::classes::slides::ALL_DECKS.iter().map(|d| d.id.to_string()).collect());
    }

    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT deck_id FROM course_access WHERE user_id = ?",
    )
    .bind(user.id)
    .fetch_all(&mut conn)
    .await?;

    Ok(rows.into_iter().map(|r| r.0).collect())
}

/// Allows a logged-in user to purchase/unlock access to a course.
#[server]
pub async fn buy_course_access(deck_id: String) -> Result<(), ServerFnError> {
    let user = crate::auth::ssr::require_user().await?;
    let deck_id = deck_id.trim();
    if deck_id.is_empty() {
        return Err(ServerFnError::ServerError("Invalid deck id.".into()));
    }

    let mut conn = crate::db::db().await?;
    sqlx::query(
        "INSERT INTO course_access (user_id, deck_id) VALUES (?, ?)
         ON CONFLICT (user_id, deck_id) DO NOTHING",
    )
    .bind(user.id)
    .bind(deck_id)
    .execute(&mut conn)
    .await?;

    tracing::info!(event = "buy_course_access", user = %user.username, deck = %deck_id, "course access unlocked");
    Ok(())
}
