//! Gated slide media: `/slides-media/{deck_id}/{*path}` serves files from
//! `content/slides/` only to accounts that can open the deck's course.
//! Any logged-in user may also fetch `thumbs/` (video first frames) and the
//! media of slide 1, which the viewer shows as a free preview.

use std::path::{Component, Path as FsPath, PathBuf};

use axum::{
    body::Body,
    extract::{Path, Request},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use tower_http::services::ServeFile;

use super::ALL_DECKS;
use crate::classes::ALL_COURSES;

/// Deck media lives here — deliberately OUTSIDE `public/`, so the static
/// fallback can never serve it.
const SLIDES_DIR: &str = "content/slides";

pub async fn serve_slide_media(
    Path((deck_id, path)): Path<(String, String)>,
    headers: HeaderMap,
    req: Request,
) -> Response {
    // 1. Known deck + its course, and a safe relative path under media/ or thumbs/.
    let Some(deck) = ALL_DECKS.iter().find(|deck| deck.id == deck_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(course) = ALL_COURSES
        .iter()
        .find(|course| course.slide_decks.contains(&deck_id.as_str()))
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let rel = FsPath::new(&path);
    let safe = (rel.starts_with("media") || rel.starts_with("thumbs"))
        && rel.components().all(|c| matches!(c, Component::Normal(_)));
    if !safe {
        return StatusCode::NOT_FOUND.into_response();
    }

    // 2. Logged in?
    let Some((user_id, role)) = user_from_headers(&headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    // 3. Previews are open to every logged-in user; the rest needs the course.
    let is_preview = rel.starts_with("thumbs")
        || deck.slides.first().is_some_and(|slide| {
            slide
                .media
                .iter()
                .any(|m| m.src == path || m.poster == Some(path.as_str()))
        });
    if !is_preview {
        match can_access_course(user_id, &role, course.id).await {
            Ok(true) => {}
            Ok(false) => return StatusCode::FORBIDDEN.into_response(),
            Err(e) => {
                tracing::error!(error = %e, "slide media access check failed");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }

    // 4. Stream it. ServeFile handles MIME, Range (video seeking) and 404.
    let file: PathBuf = [SLIDES_DIR, &deck_id, &path].iter().collect();
    match ServeFile::new(file).try_call(req).await {
        Ok(res) => {
            let (mut parts, body) = res.into_parts();
            parts.headers.insert(
                header::CACHE_CONTROL,
                header::HeaderValue::from_static("private, max-age=3600"),
            );
            Response::from_parts(parts, Body::new(body))
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Session cookie -> (user_id, role). None = not logged in.
/// (Raw axum route: no Leptos request context, so read the header directly.)
async fn user_from_headers(headers: &HeaderMap) -> Option<(i64, String)> {
    let cookie = headers.get(header::COOKIE)?.to_str().ok()?;
    let token = cookie
        .split(';')
        .map(str::trim)
        .find_map(|c| c.strip_prefix("session="))?;
    let mut conn = crate::db::db().await.ok()?;
    sqlx::query_as::<_, (i64, String)>(
        "SELECT u.id, u.role FROM users u
         JOIN sessions s ON s.user_id = u.id
         WHERE s.token = ?",
    )
    .bind(token)
    .fetch_optional(&mut conn)
    .await
    .ok()
    .flatten()
}

/// Same account-level rule as `get_unlocked_courses`: admin, an active
/// subscription, or any learner on the account enrolled in the course.
async fn can_access_course(
    user_id: i64,
    role: &str,
    course_id: i64,
) -> Result<bool, leptos::prelude::ServerFnError> {
    if role == "admin" {
        return Ok(true);
    }
    let mut conn = crate::db::db().await?;
    let (allowed,): (bool,) = sqlx::query_as(
        "SELECT EXISTS (
             SELECT 1 FROM subscriptions
             WHERE user_id = ? AND paid_until > datetime('now')
         ) OR EXISTS (
             SELECT 1 FROM learner_courses lc
             JOIN learner_profiles lp ON lp.id = lc.learner_profile_id
             WHERE lc.course_id = ? AND (lp.owner_user_id = ? OR lp.linked_user_id = ?)
         )",
    )
    .bind(user_id)
    .bind(course_id)
    .bind(user_id)
    .bind(user_id)
    .fetch_one(&mut conn)
    .await?;
    Ok(allowed)
}
