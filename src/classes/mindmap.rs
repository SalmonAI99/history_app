//! Interactive week-by-week course map component.

use leptos::prelude::*;
use leptos_router::components::A;

use super::CourseWeek;

#[component]
pub fn CourseMindMap(
    course_id: i64,
    course_title: &'static str,
    weeks: &'static [CourseWeek],
) -> impl IntoView {
    let selected_week = RwSignal::new(0usize);

    view! {
        <div class="course-mindmap">
            <div class="mindmap-scroll" role="region" aria-label="Course mind map" tabindex="0">
                <div class="mindmap-canvas">
                    <svg class="mindmap-branches" viewBox="0 0 800 360" preserveAspectRatio="none" aria-hidden="true">
                        <path d="M400 180 C310 180 260 100 160 72"/>
                        <path d="M400 180 C490 180 540 100 640 72"/>
                        <path d="M400 180 C400 235 400 265 400 310"/>
                    </svg>
                    <div class="mindmap-root">
                        <span>COURSE MAP</span>
                        <strong>{course_title}</strong>
                    </div>
                    {weeks.iter().enumerate().map(|(index, week)| {
                        let number = week.number;
                        let title = week.title;
                        view! {
                            <button
                                type="button"
                                class=format!("mindmap-branch mindmap-branch-{index}")
                                class:active=move || selected_week.get() == index
                                aria-pressed=move || selected_week.get() == index
                                on:click=move |_| selected_week.set(index)
                            >
                                <span>{format!("WEEK {number}")}</span>
                                <strong>{title}</strong>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </div>

            <div class="mindmap-detail" aria-live="polite">
                {move || weeks.get(selected_week.get()).map(|week| {
                    let deck_id = week.slide_deck.to_string();
                    view! {
                        <div>
                            <p class="mindmap-detail-kicker">{format!("WEEK {} · LEARNING PATH", week.number)}</p>
                            <h3>{week.title}</h3>
                            <p>{week.summary}</p>
                            <ul>
                                {week.topics.iter().map(|topic| view! { <li>{*topic}</li> }).collect_view()}
                            </ul>
                            <A
                                href=format!("/classes/{course_id}?deck={deck_id}")
                                attr:class="mindmap-open-slides"
                            >
                                "Explore related slides →"
                            </A>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
