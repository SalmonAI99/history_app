use leptos::prelude::*;
pub mod autumn_post;
pub mod welcome_post;
use autumn_post::autumn_post;
use welcome_post::welcome_post;

pub struct Post {
    pub slug: &'static str,
    pub title: &'static str,
    pub kind: &'static str,
    pub published_at: &'static str,
    pub render: fn() -> AnyView,
}

pub const ALL_POSTS: &[Post] = &[
    Post {
        slug: "welcome",
        title: "Welcome to Book Reader",
        kind: "blog",
        published_at: "2026-10-01",
        render: welcome_post,
    },
    Post {
        slug: "autumn-courses",
        title: "Autumn Courses Registration",
        kind: "news",
        published_at: "2026-10-02",
        render: autumn_post,
    },
];

pub fn get_posts(kind: &str) -> Vec<&'static Post> {
    let mut posts: Vec<_> = ALL_POSTS.iter().filter(|p| kind == "all" || p.kind == kind).collect();
    posts.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    posts
}

pub fn get_recent_posts() -> Vec<&'static Post> {
    let mut posts: Vec<_> = ALL_POSTS.iter().collect();
    posts.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    posts.truncate(3);
    posts
}

pub fn get_post(slug: &str) -> Option<&'static Post> {
    ALL_POSTS.iter().find(|p| p.slug == slug)
}

