//! Course catalog, course data types, and comment server functions.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{
    components::A,
    hooks::{use_params_map, use_query_map},
};
use serde::{Deserialize, Serialize};

pub mod mindmap;
pub mod learning;
pub mod middle_east;
pub mod slides;
pub mod toilet;
pub mod view;
pub mod videos;

use crate::auth::User;
use self::slides::{SlideDeckViewer, ALL_DECKS};
use self::mindmap::CourseMindMap;
use self::videos::{collect_course_videos, CourseVideoLibrary};
use self::view::{ClassDetailPage, ClassesPage};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ClassComment {
    pub id: i64,
    pub username: String,
    pub content: String,
    pub created_at: String,
}


#[server]
pub async fn get_class_comments(class_id: i64) -> Result<Vec<ClassComment>, ServerFnError> {
    crate::auth::ssr::require_user().await?;
    let mut conn = crate::db::db().await?;
    Ok(sqlx::query_as("SELECT cc.id, u.username, cc.content, cc.created_at FROM class_comments cc JOIN users u ON u.id = cc.user_id WHERE cc.class_id = ? ORDER BY cc.created_at, cc.id")
        .bind(class_id).fetch_all(&mut conn).await?)
}

#[server]
pub async fn add_class_comment(class_id: i64, content: String) -> Result<(), ServerFnError> {
    let user = crate::auth::ssr::require_user().await?;
    let content = content.trim();
    if content.is_empty() || content.len() > 2000 {
        return Err(ServerFnError::ServerError("Comment must be 1–2,000 characters.".into()));
    }
    let mut conn = crate::db::db().await?;
    sqlx::query("INSERT INTO class_comments (class_id, user_id, content) VALUES (?, ?, ?)")
        .bind(class_id).bind(user.id).bind(content).execute(&mut conn).await?;
    Ok(())
}

fn course_thumbnail(course: &Course) -> Option<String> {
    (!course.thumbnail.is_empty()).then(|| course.thumbnail.to_string())
}



use middle_east::WEEKS;

pub const COURSE_MIDDLE_EAST: Course = Course {
    id: 1,
    title: "Middle East History",
    thumbnail: "/images/middle-east-course.jpg",
    notes: include_str!("middle_east/notes.md"),
    syllabus: include_str!("middle_east/description.md"),
    content: include_str!("middle_east/course.md"),
    video_intro: include_str!("middle_east/video_intro.md"),
    slide_decks: &["israel_palestine_1", "israel_palestine_2"],
    weeks: WEEKS,
};

pub const COURSE_TOILET: Course = Course {
    id: 2,
    title: "toilet",
    thumbnail: "/images/Fixing_toilet_in_ancient_times_20261004222952.jpg",
    notes: include_str!("middle_east/notes.md"),
    syllabus: include_str!("middle_east/description.md"),
    content: include_str!("middle_east/course.md"),
    video_intro: include_str!("middle_east/video_intro.md"),
    slide_decks: &["toilet14"],
    weeks: TOILET_WEEKS,
};

pub const TOILET_WEEKS: &[CourseWeek] = &[CourseWeek {
    number: 1,
    title: "Toilet history",
    summary: "Explore the history and development of toilets through the lesson deck.",
    topics: &["Toilet history and development"],
    slide_deck: "toilet14",
}];

pub const ALL_COURSES: &[Course] = &[COURSE_MIDDLE_EAST, COURSE_TOILET];


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Course {
    pub id: i64,
    pub thumbnail: &'static str,
    pub title: &'static str,
    pub notes: &'static str,
    pub syllabus: &'static str,
    pub content: &'static str,
    pub video_intro: &'static str,
    pub slide_decks: &'static [&'static str],
    pub weeks: &'static [CourseWeek],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CourseWeek {
    pub number: usize,
    pub title: &'static str,
    pub summary: &'static str,
    pub topics: &'static [&'static str],
    pub slide_deck: &'static str,
}
