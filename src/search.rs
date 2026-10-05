#[cfg(feature = "ssr")]
use std::collections::HashSet;

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
use crate::classes::ALL_COURSES;
#[cfg(feature = "ssr")]
use crate::classes::slides::{MediaType, ALL_DECKS};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub category: String,
    pub excerpt: String,
    pub href: String,
}

#[cfg(feature = "ssr")]
struct SearchCandidate {
    title: String,
    category: String,
    body: String,
    fallback_excerpt: String,
    href: String,
}

#[cfg(feature = "ssr")]
struct RankedResult {
    score: u8,
    result: SearchResult,
}

#[server]
pub async fn search_content(query: String) -> Result<Vec<SearchResult>, ServerFnError> {
    let query: String = query.trim().chars().take(120).collect();
    let normalized_query = query.to_lowercase();
    let terms: Vec<String> = normalized_query
        .split_whitespace()
        .map(str::to_string)
        .collect();
    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let mut candidates = Vec::new();

    for post in crate::posts::ALL_POSTS {
        candidates.push(SearchCandidate {
            title: post.title.to_string(),
            category: if post.kind == "news" { "News" } else { "Blog" }.to_string(),
            body: format!("{} {}", post.slug, post.search_text),
            fallback_excerpt: format!("{} post", post.kind),
            href: format!("/{}/{}", post.kind, post.slug),
        });
    }

    let mut conn = crate::db::db().await?;
    let books: Vec<(i64, String, String)> =
        sqlx::query_as("SELECT id, title, author FROM books ORDER BY id")
            .fetch_all(&mut conn)
            .await?;
    for (id, title, author) in books {
        candidates.push(SearchCandidate {
            title: title.clone(),
            category: "Book".to_string(),
            body: author,
            fallback_excerpt: "Open this book in the library.".to_string(),
            href: format!("/book/{id}/chapter/1"),
        });
    }

    let chapters: Vec<(i64, String, i64, String, String)> = sqlx::query_as(
        "SELECT c.book_id, b.title, c.idx, c.title, c.content
         FROM chapters c JOIN books b ON b.id = c.book_id
         ORDER BY c.book_id, c.idx",
    )
    .fetch_all(&mut conn)
    .await?;
    for (book_id, book_title, chapter, title, content) in chapters {
        candidates.push(SearchCandidate {
            title: format!("{book_title} · Chapter {chapter}: {title}"),
            category: "Chapter".to_string(),
            body: content,
            fallback_excerpt: format!("Chapter {chapter} of {book_title}."),
            href: format!("/book/{book_id}/chapter/{chapter}"),
        });
    }

    // Class pages require a login, so don't return their descriptions to guests.
    let is_logged_in = crate::auth::ssr::current_user()
        .await
        .unwrap_or(None)
        .is_some();
    if is_logged_in {
        for course in ALL_COURSES {
            let mut body = [
                course.notes,
                course.syllabus,
                course.content,
                course.video_intro,
            ]
            .join(" ");
            for week in course.weeks {
                body.push(' ');
                body.push_str(week.title);
                body.push(' ');
                body.push_str(week.summary);
                body.push(' ');
                body.push_str(&week.topics.join(" "));
            }
            candidates.push(SearchCandidate {
                title: course.title.to_string(),
                category: "Course".to_string(),
                body,
                fallback_excerpt: "Course overview and interactive syllabus.".to_string(),
                href: format!("/classes/{}", course.id),
            });
        }
    }

    let unlocked_decks: HashSet<String> = crate::classes::slides::get_unlocked_courses()
        .await
        .unwrap_or_default()
        .into_iter()
        .collect();
    for deck in ALL_DECKS {
        let deck_href = ALL_COURSES
            .iter()
            .find(|course| course.slide_decks.contains(&deck.id))
            .map(|course| format!("/classes/{}?deck={}", course.id, deck.id))
            .unwrap_or_else(|| format!("/slides/{}", deck.id));

        candidates.push(SearchCandidate {
            title: deck.title.to_string(),
            category: "Lesson deck".to_string(),
            body: format!("{} slides", deck.slide_count),
            fallback_excerpt: format!("{} slides in this lesson deck.", deck.slide_count),
            href: deck_href.clone(),
        });

        let is_unlocked = unlocked_decks.contains(deck.id);
        for slide in deck.slides {
            // Locked decks only expose their first slide as a free preview.
            if !is_unlocked && slide.num != 1 {
                continue;
            }

            let mut body = slide.paragraphs.join(" ");
            for media in slide.media {
                if let Some(caption) = media.caption {
                    if !is_hash_caption(caption) {
                        body.push(' ');
                        body.push_str(caption);
                    }
                }
            }

            let category = if slide
                .media
                .iter()
                .any(|media| media.media_type == MediaType::Video)
            {
                "Lesson video"
            } else {
                "Lesson slide"
            };
            candidates.push(SearchCandidate {
                title: format!("{} · Slide {}: {}", deck.title, slide.num, slide.title),
                category: category.to_string(),
                body,
                fallback_excerpt: format!("Slide {} in {}.", slide.num, deck.title),
                href: format!("{}#slide-{}", deck_href, slide.num),
            });
        }
    }

    // Uploaded file names are visible only to logged-in users, just like the
    // existing materials pages. The download endpoint checks access again.
    if let Ok(files) = crate::files::list_files().await {
        for file in files {
            let category = if file.is_video() { "Video" } else { "Material" };
            let fallback_excerpt = format!("{} · {}", category, file.size_label());
            let href = file.href();
            candidates.push(SearchCandidate {
                title: file.original_name,
                category: category.to_string(),
                body: file.mime_type,
                fallback_excerpt,
                href,
            });
        }
    }

    let mut results: Vec<RankedResult> = candidates
        .into_iter()
        .filter_map(|candidate| rank_candidate(candidate, &normalized_query, &terms))
        .collect();
    results.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.result.title.cmp(&right.result.title))
    });

    Ok(results
        .into_iter()
        .take(30)
        .map(|ranked| ranked.result)
        .collect())
}

#[cfg(feature = "ssr")]
fn is_hash_caption(caption: &str) -> bool {
    caption.len() >= 24
        && caption
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

#[cfg(feature = "ssr")]
fn rank_candidate(
    candidate: SearchCandidate,
    query: &str,
    terms: &[String],
) -> Option<RankedResult> {
    let title = candidate.title.to_lowercase();
    let body = candidate.body.to_lowercase();
    if !terms
        .iter()
        .all(|term| title.contains(term) || body.contains(term))
    {
        return None;
    }

    let score = if title == query {
        100
    } else if title.contains(query) {
        80
    } else if terms.iter().all(|term| title.contains(term)) {
        60
    } else {
        30
    };
    let excerpt = candidate
        .body
        .split(|character| matches!(character, '\n' | '。' | '.' | '！' | '？'))
        .find(|segment| {
            let segment = segment.to_lowercase();
            terms.iter().any(|term| segment.contains(term))
        })
        .map(truncate_excerpt)
        .filter(|excerpt| !excerpt.is_empty())
        .unwrap_or(candidate.fallback_excerpt);

    Some(RankedResult {
        score,
        result: SearchResult {
            title: candidate.title,
            category: candidate.category,
            excerpt,
            href: candidate.href,
        },
    })
}

#[cfg(feature = "ssr")]
fn truncate_excerpt(text: &str) -> String {
    let excerpt: String = text.trim().chars().take(180).collect();
    if text.trim().chars().count() > 180 {
        format!("{excerpt}…")
    } else {
        excerpt
    }
}

#[component]
pub fn SearchPage() -> impl IntoView {
    let query_map = use_query_map();
    let resource_query = query_map.clone();
    let display_query = query_map;
    let results = Resource::new(
        move || resource_query.read().get("q").unwrap_or_default(),
        move |query| async move { search_content(query).await },
    );

    view! {
        <Title text="Search"/>
        <section class="search-page">
            <p class="search-eyebrow">"GLOBAL SEARCH"</p>
            <h2>"Search the library"</h2>
            <p class="search-description">"Search books, chapters, courses, lesson slides, videos, and articles."</p>
            <Transition fallback=|| view! { <p class="search-status">"Searching…"</p> }>
                {move || {
                    let query = display_query.read().get("q").unwrap_or_default();
                    if query.trim().is_empty() {
                        view! { <p class="search-status">"Enter a title, topic, or phrase to search."</p> }.into_any()
                    } else {
                        match results.get() {
                            None => ().into_any(),
                            Some(Err(_)) => view! { <p class="search-status">"Search is temporarily unavailable."</p> }.into_any(),
                            Some(Ok(found)) if found.is_empty() => view! {
                                <p class="search-status">"No results found. Try another title or phrase."</p>
                            }.into_any(),
                            Some(Ok(found)) => view! {
                                <p class="search-count">{format!("{} results for “{}”", found.len(), query)}</p>
                                <ul class="search-results">
                                    {found.into_iter().map(|result| view! {
                                        <li class="search-result">
                                            <a href=result.href>
                                                <span class="search-result-category">{result.category}</span>
                                                <strong>{result.title}</strong>
                                                <span class="search-result-excerpt">{result.excerpt}</span>
                                            </a>
                                        </li>
                                    }).collect_view()}
                                </ul>
                            }.into_any(),
                        }
                    }
                }}
            </Transition>
        </section>
    }
}
