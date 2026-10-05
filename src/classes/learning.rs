//! Learner profiles, course assignments, and course progress.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::Connection;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct LearnerProfile {
    pub id: i64,
    pub name: String,
    pub age_range: String,
    pub gender: String,
    pub account_username: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CourseLearningProgress {
    pub course_id: i64,
    pub title: String,
    pub current_deck: Option<String>,
    pub current_slide: i64,
    pub total_slides: i64,
    pub progress_percent: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LearnerDashboard {
    pub learner: LearnerProfile,
    pub courses: Vec<CourseLearningProgress>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LearningDashboard {
    pub learners: Vec<LearnerDashboard>,
    pub purchased_course_ids: Vec<i64>,
}

#[server]
pub async fn get_learning_dashboard() -> Result<LearningDashboard, ServerFnError> {
    let user = crate::auth::ssr::require_user().await?;
    let mut conn = crate::db::db().await?;

    let learner_query = if user.role == "student" {
        "SELECT lp.id, lp.name, lp.age_range, lp.gender, linked.username AS account_username
         FROM learner_profiles lp
         LEFT JOIN users linked ON linked.id = lp.linked_user_id
         WHERE lp.owner_user_id = ? OR lp.linked_user_id = ? ORDER BY lp.id"
    } else {
        "SELECT lp.id, lp.name, lp.age_range, lp.gender, linked.username AS account_username
         FROM learner_profiles lp
         LEFT JOIN users linked ON linked.id = lp.linked_user_id
         WHERE lp.owner_user_id = ? ORDER BY lp.id"
    };
    let mut learner_query = sqlx::query_as::<_, LearnerProfile>(learner_query).bind(user.id);
    if user.role == "student" {
        learner_query = learner_query.bind(user.id);
    }
    let learners: Vec<LearnerProfile> = learner_query
    .fetch_all(&mut conn)
    .await?;

    let purchased_course_ids: Vec<(i64,)> = sqlx::query_as(
        "SELECT course_id FROM course_purchases WHERE user_id = ? ORDER BY course_id",
    )
    .bind(user.id)
    .fetch_all(&mut conn)
    .await?;

    let mut dashboards = Vec::with_capacity(learners.len());
    for learner in learners {
        let course_ids: Vec<(i64,)> = sqlx::query_as(
            "SELECT course_id FROM learner_courses
             WHERE learner_profile_id = ? ORDER BY course_id",
        )
        .bind(learner.id)
        .fetch_all(&mut conn)
        .await?;

        let mut courses = Vec::new();
        for (course_id,) in course_ids {
            let Some(course) = crate::classes::ALL_COURSES.iter().find(|c| c.id == course_id) else {
                continue;
            };
            let deck_ids = course.slide_decks;
            let total_slides: i64 = crate::classes::slides::ALL_DECKS
                .iter()
                .filter(|deck| deck_ids.contains(&deck.id))
                .map(|deck| deck.slide_count as i64)
                .sum();

            let progress_rows: Vec<(String, i64, String)> = sqlx::query_as(
                "SELECT deck_id, last_slide, updated_at FROM course_progress
                 WHERE learner_profile_id = ? AND course_id = ?
                 ORDER BY updated_at DESC",
            )
            .bind(learner.id)
            .bind(course_id)
            .fetch_all(&mut conn)
            .await?;

            let current_deck = progress_rows.first().map(|row| row.0.clone());
            let current_slide = progress_rows.first().map(|row| row.1).unwrap_or(0);
            let completed_slide_count: i64 = progress_rows
                .iter()
                .map(|(deck_id, slide, _)| {
                    let deck_total = crate::classes::slides::ALL_DECKS
                        .iter()
                        .find(|deck| deck.id == deck_id)
                        .map(|deck| deck.slide_count as i64)
                        .unwrap_or(0);
                    (*slide).clamp(0, deck_total)
                })
                .sum();
            let progress_percent = if total_slides == 0 {
                0
            } else {
                (completed_slide_count * 100 / total_slides).clamp(0, 100)
            };

            courses.push(CourseLearningProgress {
                course_id,
                title: course.title.to_string(),
                current_deck,
                current_slide,
                total_slides,
                progress_percent,
            });
        }
        dashboards.push(LearnerDashboard { learner, courses });
    }

    Ok(LearningDashboard {
        learners: dashboards,
        purchased_course_ids: purchased_course_ids.into_iter().map(|row| row.0).collect(),
    })
}

#[server]
pub async fn create_child_account(
    name: String,
    age_range: String,
    gender: String,
    username: String,
    password: String,
) -> Result<(), ServerFnError> {
    let user = crate::auth::ssr::require_user().await?;
    if user.role != "parent" {
        return Err(ServerFnError::ServerError("Only parent accounts can add another learner.".into()));
    }
    let name = name.trim();
    validate_profile(name, &age_range, &gender)?;
    let username = username.trim();
    validate_username(username)?;
    if password.len() < 6 {
        return Err(ServerFnError::ServerError("Child password needs at least 6 characters.".into()));
    }
    let password_hash = match bcrypt::hash(&password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => return Err(ServerFnError::ServerError("Could not create the child account.".into())),
    };

    let mut conn = crate::db::db().await?;
    let mut tx = conn.begin().await?;
    let child = sqlx::query("INSERT INTO users (username, password_hash, role) VALUES (?, ?, 'student')")
        .bind(username)
        .bind(password_hash)
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
    .bind(user.id)
    .bind(child.last_insert_rowid())
    .bind(name)
    .bind(age_range)
    .bind(gender)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// A student creates a short-lived token and shares it with a parent.
#[server]
pub async fn generate_parent_link_token() -> Result<String, ServerFnError> {
    let user = crate::auth::ssr::require_user().await?;
    if user.role != "student" {
        return Err(ServerFnError::ServerError("Only student accounts can create a parent link code.".into()));
    }
    let mut conn = crate::db::db().await?;
    let profile: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM learner_profiles
         WHERE owner_user_id = ? AND linked_user_id IS NULL
         ORDER BY id LIMIT 1",
    )
    .bind(user.id)
    .fetch_optional(&mut conn)
    .await?;
    if profile.is_none() {
        return Err(ServerFnError::ServerError("This account is already linked to a parent, or has no learner profile.".into()));
    }

    let token: String = sqlx::query_scalar("SELECT lower(hex(randomblob(24)))")
        .fetch_one(&mut conn)
        .await?;
    sqlx::query(
        "INSERT INTO account_link_tokens (token, learner_user_id, expires_at)
         VALUES (?, ?, datetime('now', '+30 minutes'))",
    )
    .bind(&token)
    .bind(user.id)
    .execute(&mut conn)
    .await?;
    Ok(token)
}

/// A parent links a student's existing profile using that student's one-time code.
#[server]
pub async fn link_existing_child(token: String, learner_id: i64) -> Result<(), ServerFnError> {
    let user = crate::auth::ssr::require_user().await?;
    if user.role != "parent" {
        return Err(ServerFnError::ServerError("Only parent accounts can link a student account.".into()));
    }
    let token = token.trim();
    if token.len() != 48 || !token.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(ServerFnError::ServerError("Enter the 30-minute link code from the student's account.".into()));
    }

    let mut conn = crate::db::db().await?;
    let mut tx = conn.begin().await?;
    attach_student_profile(
        &mut *tx,
        user.id,
        token,
        (learner_id > 0).then_some(learner_id),
        "",
        "",
        "",
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(feature = "ssr")]
pub(crate) async fn attach_student_profile(
    conn: &mut sqlx::SqliteConnection,
    parent_id: i64,
    token: &str,
    target_profile_id: Option<i64>,
    fallback_name: &str,
    fallback_age_range: &str,
    fallback_gender: &str,
) -> Result<(), ServerFnError> {
    let linked_user: Option<(i64, String)> = sqlx::query_as(
        "SELECT t.learner_user_id, u.username FROM account_link_tokens t
         JOIN users u ON u.id = t.learner_user_id
         WHERE t.token = ? AND t.used_at IS NULL AND t.expires_at > datetime('now') AND u.role = 'student'",
    )
    .bind(token)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((child_id, child_username)) = linked_user else {
        return Err(ServerFnError::ServerError("That link code is invalid, expired, or already used.".into()));
    };

    let already_linked: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM learner_profiles WHERE linked_user_id = ?",
    )
    .bind(child_id)
    .fetch_optional(&mut *conn)
    .await?;
    if already_linked.is_some() {
        return Err(ServerFnError::ServerError("That student account is already linked to a parent.".into()));
    }

    let target_profile = if let Some(target_id) = target_profile_id {
        let target: Option<(String, String, String)> = sqlx::query_as(
            "SELECT name, age_range, gender FROM learner_profiles
             WHERE id = ? AND owner_user_id = ? AND linked_user_id IS NULL",
        )
        .bind(target_id)
        .bind(parent_id)
        .fetch_optional(&mut *conn)
        .await?;
        let Some(target) = target else {
            return Err(ServerFnError::ServerError("Choose an unlinked learner profile.".into()));
        };
        Some(target)
    } else {
        None
    };

    let child_profile: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM learner_profiles WHERE owner_user_id = ? ORDER BY id LIMIT 1",
    )
    .bind(child_id)
    .fetch_optional(&mut *conn)
    .await?;
    if let Some((source_id,)) = child_profile {
        if let Some(target_id) = target_profile_id.filter(|id| *id != source_id) {
            let (name, age_range, gender) = target_profile.expect("validated target profile");
            sqlx::query(
                "INSERT OR IGNORE INTO learner_courses (learner_profile_id, course_id)
                 SELECT ?, course_id FROM learner_courses WHERE learner_profile_id = ?",
            )
            .bind(source_id)
            .bind(target_id)
            .execute(&mut *conn)
            .await?;
            sqlx::query(
                "INSERT INTO course_progress (learner_profile_id, course_id, deck_id, last_slide, updated_at)
                 SELECT ?, course_id, deck_id, last_slide, updated_at FROM course_progress WHERE learner_profile_id = ?
                 ON CONFLICT (learner_profile_id, course_id, deck_id) DO UPDATE SET
                     last_slide = MAX(course_progress.last_slide, excluded.last_slide),
                     updated_at = MAX(course_progress.updated_at, excluded.updated_at)",
            )
            .bind(source_id)
            .bind(target_id)
            .execute(&mut *conn)
            .await?;
            sqlx::query("DELETE FROM learner_courses WHERE learner_profile_id = ?")
                .bind(target_id)
                .execute(&mut *conn)
                .await?;
            sqlx::query("DELETE FROM course_progress WHERE learner_profile_id = ?")
                .bind(target_id)
                .execute(&mut *conn)
                .await?;
            sqlx::query(
                "UPDATE learner_profiles SET owner_user_id = ?, linked_user_id = ?, name = ?, age_range = ?, gender = ?
                 WHERE id = ? AND linked_user_id IS NULL",
            )
            .bind(parent_id)
            .bind(child_id)
            .bind(name)
            .bind(age_range)
            .bind(gender)
            .bind(source_id)
            .execute(&mut *conn)
            .await?;
            sqlx::query("DELETE FROM learner_profiles WHERE id = ? AND owner_user_id = ?")
                .bind(target_id)
                .bind(parent_id)
                .execute(&mut *conn)
                .await?;
        } else {
            sqlx::query(
                "UPDATE learner_profiles SET owner_user_id = ?, linked_user_id = ? WHERE id = ?",
            )
            .bind(parent_id)
            .bind(child_id)
            .bind(source_id)
            .execute(&mut *conn)
            .await?;
        }
    } else if let Some((profile_id,)) = child_profile {
        sqlx::query(
            "UPDATE learner_profiles SET owner_user_id = ?, linked_user_id = ?
             WHERE id = ? AND linked_user_id IS NULL",
        )
        .bind(parent_id)
        .bind(child_id)
        .bind(profile_id)
        .execute(&mut *conn)
        .await?;
    } else if let Some(target_id) = target_profile_id {
        sqlx::query("UPDATE learner_profiles SET linked_user_id = ? WHERE id = ? AND owner_user_id = ?")
            .bind(child_id)
            .bind(target_id)
            .bind(parent_id)
            .execute(&mut *conn)
            .await?;
    } else {
        sqlx::query(
            "INSERT INTO learner_profiles (owner_user_id, linked_user_id, name, age_range, gender)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(parent_id)
        .bind(child_id)
        .bind(if fallback_name.is_empty() { &child_username } else { fallback_name })
        .bind(fallback_age_range)
        .bind(fallback_gender)
        .execute(&mut *conn)
        .await?;
    }

    let consumed = sqlx::query(
        "UPDATE account_link_tokens SET used_at = datetime('now')
         WHERE token = ? AND used_at IS NULL AND expires_at > datetime('now')",
    )
    .bind(token)
    .execute(&mut *conn)
    .await?;
    if consumed.rows_affected() != 1 {
        return Err(ServerFnError::ServerError("That link code has already been used.".into()));
    }
    Ok(())
}

#[server]
pub async fn assign_course_to_learner(
    learner_id: i64,
    course_id: i64,
) -> Result<(), ServerFnError> {
    let user = crate::auth::ssr::require_user().await?;
    if user.role != "parent" {
        return Err(ServerFnError::ServerError("Only parent accounts can assign courses.".into()));
    }
    if !crate::classes::ALL_COURSES.iter().any(|course| course.id == course_id) {
        return Err(ServerFnError::ServerError("Course not found.".into()));
    }

    let mut conn = crate::db::db().await?;
    let owned_profile: Option<(i64,)> = sqlx::query_as(
        "SELECT id FROM learner_profiles WHERE id = ? AND owner_user_id = ?",
    )
    .bind(learner_id)
    .bind(user.id)
    .fetch_optional(&mut conn)
    .await?;
    if owned_profile.is_none() {
        return Err(ServerFnError::ServerError("Learner profile not found.".into()));
    }

    let purchased: Option<(i64,)> = sqlx::query_as(
        "SELECT course_id FROM course_purchases WHERE user_id = ? AND course_id = ?",
    )
    .bind(user.id)
    .bind(course_id)
    .fetch_optional(&mut conn)
    .await?;
    if purchased.is_none() {
        return Err(ServerFnError::ServerError("This course has not been purchased by your account yet.".into()));
    }

    sqlx::query(
        "INSERT OR IGNORE INTO learner_courses (learner_profile_id, course_id) VALUES (?, ?)",
    )
    .bind(learner_id)
    .bind(course_id)
    .execute(&mut conn)
    .await?;
    Ok(())
}

#[server]
pub async fn record_course_progress(
    learner_id: i64,
    course_id: i64,
    deck_id: String,
    slide_num: i64,
) -> Result<(), ServerFnError> {
    let user = crate::auth::ssr::require_user().await?;
    let Some(course) = crate::classes::ALL_COURSES
        .iter()
        .find(|course| course.id == course_id)
    else {
        return Err(ServerFnError::ServerError("Course not found.".into()));
    };
    let Some(deck) = crate::classes::slides::ALL_DECKS.iter().find(|deck| deck.id == deck_id) else {
        return Err(ServerFnError::ServerError("Lesson deck not found.".into()));
    };
    if !course.slide_decks.contains(&deck.id) || slide_num < 1 || slide_num > deck.slide_count as i64 {
        return Err(ServerFnError::ServerError("Invalid lesson progress.".into()));
    }

    let Some(profile_id) = authorized_profile_id(&user, learner_id).await? else {
        // Administrators can preview everything; they do not need learner progress.
        return Ok(());
    };

    let mut conn = crate::db::db().await?;
    let enrolled: Option<(i64,)> = sqlx::query_as(
        "SELECT course_id FROM learner_courses WHERE learner_profile_id = ? AND course_id = ?",
    )
    .bind(profile_id)
    .bind(course_id)
    .fetch_optional(&mut conn)
    .await?;
    if enrolled.is_none() {
        return Err(ServerFnError::ServerError("This learner is not enrolled in the course.".into()));
    }

    sqlx::query(
        "INSERT INTO course_progress (learner_profile_id, course_id, deck_id, last_slide)
             VALUES (?, ?, ?, ?)
             ON CONFLICT (learner_profile_id, course_id, deck_id) DO UPDATE SET
             last_slide = MAX(course_progress.last_slide, excluded.last_slide),
             updated_at = strftime('%Y-%m-%d %H:%M:%f', 'now')",
    )
    .bind(profile_id)
    .bind(course_id)
    .bind(deck_id)
    .bind(slide_num)
    .execute(&mut conn)
    .await?;
    Ok(())
}

/// Resolve the learner requested in the URL and verify account ownership.
#[cfg(feature = "ssr")]
pub(crate) async fn authorized_profile_id(
    user: &crate::auth::User,
    learner_id: i64,
) -> Result<Option<i64>, ServerFnError> {
    if user.role == "admin" {
        return Ok(None);
    }

    let mut conn = crate::db::db().await?;
    if user.role == "student" {
        let profile: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM learner_profiles
             WHERE owner_user_id = ? OR linked_user_id = ? ORDER BY id LIMIT 1",
        )
        .bind(user.id)
        .bind(user.id)
        .fetch_optional(&mut conn)
        .await?;
        return profile
            .map(|row| Some(row.0))
            .ok_or_else(|| ServerFnError::ServerError("Learner profile not found.".into()));
    }

    if user.role == "parent" && learner_id <= 0 {
        let profile: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM learner_profiles
             WHERE owner_user_id = ? OR linked_user_id = ? ORDER BY id LIMIT 1",
        )
        .bind(user.id)
        .bind(user.id)
        .fetch_optional(&mut conn)
        .await?;
        return profile
            .map(|row| Some(row.0))
            .ok_or_else(|| ServerFnError::ServerError("Add a learner profile first.".into()));
    }

    if user.role == "parent" && learner_id > 0 {
        let profile: Option<(i64,)> = sqlx::query_as(
            "SELECT id FROM learner_profiles WHERE id = ? AND owner_user_id = ?",
        )
        .bind(learner_id)
        .bind(user.id)
        .fetch_optional(&mut conn)
        .await?;
        return profile
            .map(|row| Some(row.0))
            .ok_or_else(|| ServerFnError::ServerError("Learner profile not found.".into()));
    }

    Err(ServerFnError::ServerError("Choose a learner profile first.".into()))
}

#[cfg(feature = "ssr")]
fn validate_profile(name: &str, age_range: &str, gender: &str) -> Result<(), ServerFnError> {
    if name.is_empty() || name.chars().count() > 80 {
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

#[cfg(feature = "ssr")]
fn validate_username(username: &str) -> Result<(), ServerFnError> {
    if username.chars().count() < 3 || username.chars().count() > 64 {
        return Err(ServerFnError::ServerError("Username needs 3–64 characters.".into()));
    }
    Ok(())
}
