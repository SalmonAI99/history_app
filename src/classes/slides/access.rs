//! Course purchase checks and prototype purchase actions.

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

/// Checks course-level enrollment for the learner selected in the page URL.
#[server]
pub async fn check_course_access(
    deck_id: String,
    learner_id: i64,
) -> Result<CourseAccessStatus, ServerFnError> {
    let Some(user) = crate::auth::ssr::current_user().await? else {
        return Ok(CourseAccessStatus::default());
    };
    if user.role == "admin" {
        return Ok(CourseAccessStatus {
            is_unlocked: true,
            is_admin: true,
            is_logged_in: true,
            username: Some(user.username),
        });
    }

    let Some(course) = crate::classes::ALL_COURSES
        .iter()
        .find(|course| course.slide_decks.contains(&deck_id.as_str()))
    else {
        return Err(ServerFnError::ServerError("Course not found for this lesson.".into()));
    };
    let Some(profile_id) = crate::classes::learning::authorized_profile_id(&user, learner_id).await? else {
        return Ok(CourseAccessStatus {
            is_unlocked: true,
            is_admin: true,
            is_logged_in: true,
            username: Some(user.username),
        });
    };

    let mut conn = crate::db::db().await?;
    let enrolled: Option<(i64,)> = sqlx::query_as(
        "SELECT course_id FROM learner_courses WHERE learner_profile_id = ? AND course_id = ?",
    )
    .bind(profile_id)
    .bind(course.id)
    .fetch_optional(&mut conn)
    .await?;

    let has_subscription: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT paid_until FROM subscriptions WHERE user_id = ? AND paid_until > datetime('now')",
    )
    .bind(user.id)
    .fetch_optional(&mut conn)
    .await?;

    Ok(CourseAccessStatus {
        is_unlocked: enrolled.is_some() || has_subscription.is_some(),
        is_admin: false,
        is_logged_in: true,
        username: Some(user.username),
    })
}

/// Returns all lesson deck IDs assigned to any learner on this account.
#[server]
pub async fn get_unlocked_courses() -> Result<Vec<String>, ServerFnError> {
    let Some(user) = crate::auth::ssr::current_user().await? else {
        return Ok(Vec::new());
    };
    if user.role == "admin" {
        return Ok(crate::classes::slides::ALL_DECKS.iter().map(|d| d.id.to_string()).collect());
    }

    let mut conn = crate::db::db().await?;
    let has_subscription: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT paid_until FROM subscriptions WHERE user_id = ? AND paid_until > datetime('now')",
    )
    .bind(user.id)
    .fetch_optional(&mut conn)
    .await?;
    if has_subscription.is_some() {
        return Ok(crate::classes::slides::ALL_DECKS.iter().map(|d| d.id.to_string()).collect());
    }

    let course_ids: Vec<(i64,)> = sqlx::query_as(
         "SELECT DISTINCT lc.course_id FROM learner_courses lc
         JOIN learner_profiles lp ON lp.id = lc.learner_profile_id
         WHERE lp.owner_user_id = ? OR lp.linked_user_id = ?",
    )
    .bind(user.id)
    .bind(user.id)
    .fetch_all(&mut conn)
    .await?;

    Ok(crate::classes::ALL_COURSES
        .iter()
        .filter(|course| course_ids.iter().any(|row| row.0 == course.id))
        .flat_map(|course| course.slide_decks.iter().map(|id| id.to_string()))
        .collect())
}

/// Prototype purchase: records one course purchase and assigns it to a learner.
#[server]
pub async fn buy_course_access(
    deck_id: String,
    learner_id: i64,
) -> Result<(), ServerFnError> {
    let user = crate::auth::ssr::require_user().await?;
    let Some(course) = crate::classes::ALL_COURSES
        .iter()
        .find(|course| course.slide_decks.contains(&deck_id.as_str()))
    else {
        return Err(ServerFnError::ServerError("Course not found for this lesson.".into()));
    };
    let Some(profile_id) = crate::classes::learning::authorized_profile_id(&user, learner_id).await? else {
        return Ok(());
    };

    let mut conn = crate::db::db().await?;
    sqlx::query(
        "INSERT INTO course_purchases (user_id, course_id) VALUES (?, ?)
         ON CONFLICT (user_id, course_id) DO NOTHING",
    )
    .bind(user.id)
    .bind(course.id)
    .execute(&mut conn)
    .await?;
    sqlx::query(
        "INSERT INTO learner_courses (learner_profile_id, course_id) VALUES (?, ?)
         ON CONFLICT (learner_profile_id, course_id) DO NOTHING",
    )
    .bind(profile_id)
    .bind(course.id)
    .execute(&mut conn)
    .await?;

    tracing::info!(event = "buy_course_access", user = %user.username, course_id = course.id, learner_id = profile_id, "course access unlocked");
    Ok(())
}
