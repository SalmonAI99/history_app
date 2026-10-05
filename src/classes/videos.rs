//! Collects and renders videos referenced by course slide decks.

use leptos::prelude::*;

use super::slides::{MediaItem, MediaType, ALL_DECKS};

#[derive(Clone, Copy)]
pub struct CourseVideo {
    deck_id: &'static str,
    deck_title: &'static str,
    slide_number: usize,
    media: MediaItem,
}

impl CourseVideo {
    fn src(self) -> String {
        format!("/slides-media/{}/{}", self.deck_id, self.media.src)
    }

    fn poster(self) -> String {
        self.media
            .poster
            .map(|poster| format!("/slides-media/{}/{}", self.deck_id, poster))
            .unwrap_or_default()
    }

    /// First-frame thumbnail (`thumbs/<stem>.jpg`, made by the pipeline);
    /// viewable without unlocking the course.
    fn thumb(self) -> String {
        let stem = self
            .media
            .src
            .rsplit('/')
            .next()
            .and_then(|name| name.rsplit_once('.').map(|(stem, _)| stem))
            .unwrap_or(self.media.src);
        format!("/slides-media/{}/thumbs/{}.jpg", self.deck_id, stem)
    }

    fn title(self) -> String {
        match self.media.caption {
            Some(caption)
                if !(caption.len() >= 24
                    && caption
                        .chars()
                        .all(|character| character.is_ascii_hexdigit())) =>
            {
                caption.to_string()
            }
            _ => format!("Course clip · slide {}", self.slide_number),
        }
    }
}

pub fn collect_course_videos(deck_ids: &[&str]) -> Vec<CourseVideo> {
    let mut videos = Vec::new();

    for deck in ALL_DECKS.iter().filter(|deck| deck_ids.contains(&deck.id)) {
        for slide in deck.slides {
            for media in slide.media {
                if media.media_type == MediaType::Video {
                    videos.push(CourseVideo {
                        deck_id: deck.id,
                        deck_title: deck.title,
                        slide_number: slide.num,
                        media: *media,
                    });
                }
            }
        }
    }

    videos
}

#[component]
pub fn CourseVideoLibrary(
    videos: Vec<CourseVideo>,
    /// Whether this account may watch the course; locked shows thumbnails only.
    #[prop(into)]
    unlocked: Signal<bool>,
) -> impl IntoView {
    let selected = RwSignal::new(0usize);
    let player_videos = videos.clone();
    let poster_videos = videos.clone();
    let thumb_videos = videos.clone();
    let title_videos = videos.clone();
    let context_videos = videos.clone();

    view! {
        <div class="course-video-library">
            {if videos.is_empty() {
                view! { <p class="course-video-empty">"No course videos are available yet."</p> }.into_any()
            } else {
                view! {
                    <div class="course-video-player">
                        {move || {
                            let player_videos = player_videos.clone();
                            let poster_videos = poster_videos.clone();
                            if unlocked.get() {
                                view! {
                                    <video
                                        controls
                                        playsinline
                                        preload="metadata"
                                        src=move || player_videos.get(selected.get()).map(|video| video.src()).unwrap_or_default()
                                        poster=move || poster_videos.get(selected.get()).map(|video| video.poster()).unwrap_or_default()
                                    >
                                        "Your browser does not support HTML video."
                                    </video>
                                }.into_any()
                            } else {
                                let thumb_videos = thumb_videos.clone();
                                view! {
                                    <div class="course-video-locked">
                                        <img
                                            src=move || thumb_videos.get(selected.get()).map(|video| video.thumb()).unwrap_or_default()
                                            alt=""
                                        />
                                        <div class="course-video-locked-overlay">
                                            <span class="course-video-locked-icon" aria-hidden="true">"🔒"</span>
                                            <strong>"Unlock this course to watch"</strong>
                                            <a href="#course-lessons">"See purchase options"</a>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                        }}
                    </div>
                    <div class="course-video-now-playing" aria-live="polite">
                        <span>"NOW PLAYING"</span>
                        <h3>{move || title_videos.get(selected.get()).map(|video| video.title()).unwrap_or_default()}</h3>
                        <p>{move || context_videos.get(selected.get()).map(|video| format!("{} · Slide {}", video.deck_title, video.slide_number)).unwrap_or_default()}</p>
                    </div>
                }.into_any()
            }}

            <div class="course-video-playlist" role="group" aria-label="Course video playlist">
                {videos.iter().enumerate().map(|(index, video)| {
                    let title = video.title();
                    let thumb = video.thumb();
                    let deck_title = video.deck_title;
                    let slide_number = video.slide_number;
                    view! {
                        <button
                            type="button"
                            class="course-video-item"
                            class:active=move || selected.get() == index
                            aria-pressed=move || selected.get() == index
                            on:click=move |_| selected.set(index)
                        >
                            <img src=thumb alt="" loading="lazy"/>
                            <span class="course-video-item-copy">
                                <strong>{title}</strong>
                                <small>{format!("{} · Slide {}", deck_title, slide_number)}</small>
                            </span>
                            <span class="course-video-play-icon" aria-hidden="true">
                                {move || if unlocked.get() { "▶" } else { "🔒" }}
                            </span>
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
