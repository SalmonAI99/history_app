// Shared slide-deck catalog, models, access checks, and Leptos viewer components.

pub mod access;
pub mod model;

pub use access::*;
use super::middle_east::{israel_palestine_1, israel_palestine_2};
use super::toilet::toilet14;
use crate::classes::learning::RecordCourseProgress;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
pub use model::{MediaItem, MediaType, Slide, SlideDeck};

pub const ALL_DECKS: &[SlideDeck] = &[
    israel_palestine_1::DECK,
    israel_palestine_2::DECK,
    toilet14::DECK,
];

pub fn find_deck(id: &str) -> Option<SlideDeck> {
    ALL_DECKS.iter().copied().find(|d| d.id == id)
}

/// Dynamic Route component for `/slides/:id`
// #[component]
// pub fn SlideDeckPage() -> impl IntoView {
//     let params = use_params_map();
//     let deck_id = move || params.read().get("id").unwrap_or_default();
//
//     view! {
//         {move || {
//             let id = deck_id();
//             match find_deck(&id) {
//                 Some(deck) => view! {
//                     <SlideDeckViewer deck=deck />
//                 }.into_any(),
//                 None => view! {
//                     <div class="slides-not-found">
//                         <h2>"未找到该讲义"</h2>
//                         <p class="slides-not-found-copy">
//                             {format!("不存在 ID 为 \"{}\" 的讲义速读。", id)}
//                         </p>
//                         <A href="/slides" attr:class="button">"← 返回讲义速读列表"</A>
//                     </div>
//                 }.into_any(),
//             }
//         }}
//     }
// }

/// Index page for `/slides` listing all available lecture slide decks.
// #[component]
// pub fn SlidesIndexPage() -> impl IntoView {
//     let unlocked = Resource::new(|| (), |_| async { get_unlocked_courses().await.unwrap_or_default() });
//
//     view! {
//         <Title text="课程讲义速读 — 课件与速读大纲"/>
//         <div class="slides-index-page">
//             <header class="slides-index-header">
//                 <div class="slides-index-back">
//                     <A href="/classes" attr:class="slides-index-back-link">"← 返回课程资料库"</A>
//                 </div>
//                 <h1 class="slides-index-title">
//                     "📖 课程讲义速读 (Slide Lecture Notes)"
//                 </h1>
//                 <p class="slides-index-description">
//                     "配套 PPT 讲义精炼速读版与多媒体内容，支持全文检索与分章节复习。完全基于 Leptos SSR 渲染，流畅原生体验。"
//                 </p>
//             </header>
//
//             <Transition fallback=move || view! { <p class="loading">"正在加载课程列表..."</p> }>
//                 {move || match unlocked.get() {
//                     None => ().into_any(),
//                     Some(unlocked_list) => {
//                         view! {
//                             <div class="deck-grid">
//                                 {ALL_DECKS.iter().map(|deck| {
//                                     let deck_id = deck.id;
//                                     let deck_title = deck.title;
//                                     let slide_count = deck.slide_count;
//                                     let href = ALL_COURSES
//                                         .iter()
//                                         .find(|course| course.slide_decks.contains(&deck_id))
//                                         .map(|course| format!("/classes/{}?deck={}", course.id, deck_id))
//                                         .unwrap_or_else(|| format!("/slides/{}", deck_id));
//                                     let is_unlocked = unlocked_list.contains(&deck_id.to_string());
//
//                                     view! {
//                                         <div class="deck-card">
//                                             <div class="deck-card-main">
//                                                 <div class="deck-card-title">
//                                                     <span>{deck_title}</span>
//                                                     {if is_unlocked {
//                                                         view! {
//                                                             <span class="deck-access-badge deck-access-badge--unlocked">
//                                                                 "已解锁"
//                                                             </span>
//                                                         }.into_any()
//                                                     } else {
//                                                         view! {
//                                                             <span class="deck-access-badge deck-access-badge--locked">
//                                                                 "🔒 需购买 · ￥29.9"
//                                                             </span>
//                                                         }.into_any()
//                                                     }}
//                                                 </div>
//                                                 <div class="deck-card-meta">
//                                                     <span class="deck-card-count">
//                                                         {format!("{} 页讲义", slide_count)}
//                                                     </span>
//                                                     <span>{format!("ID: {}", deck_id)}</span>
//                                                     <span>"• 含多媒体视频/图片"</span>
//                                                 </div>
//                                             </div>
//                                             <div class="deck-card-action">
//                                                 <A
//                                                     href=href
//                                                     attr:class=if is_unlocked {
//                                                         "button deck-action-button deck-action-button--unlocked"
//                                                     } else {
//                                                         "button deck-action-button deck-action-button--locked"
//                                                     }
//                                                 >
//                                                     {if is_unlocked { "开始学习 →" } else { "查看讲义 (试读/购买) →" }}
//                                                 </A>
//                                             </div>
//                                         </div>
//                                     }
//                                 }).collect_view()}
//                             </div>
//                         }.into_any()
//                     }
//                 }}
//             </Transition>
//         </div>
//     }
// }

/// The core Leptos SSR Slide Viewer Component with Access Control & Locked Preview
#[component]
pub fn SlideDeckViewer(
    deck: SlideDeck,
    #[prop(default = 0)] learner_id: i64,
) -> impl IntoView {
    let deck_id = deck.id.to_string();
    let course_id = super::ALL_COURSES
        .iter()
        .find(|course| course.slide_decks.contains(&deck.id))
        .map(|course| course.id)
        .unwrap_or_default();
    let access_deck_id = deck_id.clone();
    let access = Resource::new(
        move || (access_deck_id.clone(), learner_id),
        |(id, learner_id)| async move { check_course_access(id, learner_id).await.unwrap_or_default() },
    );
    let buy_action = ServerAction::<BuyCourseAccess>::new();
    let progress_action = ServerAction::<RecordCourseProgress>::new();
    let progress_started = RwSignal::new(false);

    // Opening a lesson records its first slide; selecting a slide in the
    // inventory advances the learner's saved position.
    let first_deck_id = deck_id.clone();
    Effect::new(move |_| {
        if !progress_started.get()
            && access.get().is_some_and(|status| status.is_unlocked && !status.is_admin)
        {
            progress_started.set(true);
            progress_action.dispatch(RecordCourseProgress {
                learner_id,
                course_id,
                deck_id: first_deck_id.clone(),
                slide_num: 1,
            });
        }
    });

    // After purchase/unlock action succeeds, refetch access:
    Effect::new(move |_| {
        if matches!(buy_action.value().get(), Some(Ok(()))) {
            access.refetch();
        }
    });

    let (query, set_query) = signal(String::new());
    let page_title = format!("{} — 课程讲义速读", deck.title);

    view! {
        <Title text=page_title/>
        <div class="slide-deck-viewer">
            <Transition fallback=move || view! { <p class="loading">"正在加载讲义与权限..."</p> }>
                {move || match access.get() {
                    None => ().into_any(),
                    Some(status) => {
                        let is_unlocked = status.is_unlocked;

                        view! {
                            <div>
                                <header class="deck-header">
                                    <h1 class="deck-title">
                                        {deck.title}
                                    </h1>
                                    <div class="deck-meta">
                                        <span>{format!("📑 共 {} 页讲义", deck.slide_count)}</span>
                                        <span>"🎬 嵌入式视频/图档"</span>
                                        {if is_unlocked {
                                            if status.is_admin {
                                                view! { <span class="deck-status-badge deck-status-badge--admin">"👑 管理员授权"</span> }.into_any()
                                            } else {
                                                view! { <span class="deck-status-badge deck-status-badge--unlocked">"✅ 已解锁完整讲义"</span> }.into_any()
                                            }
                                        } else {
                                            view! {
                                                <span class="deck-status-badge deck-status-badge--locked">
                                                    "🔒 未解锁 (试读模式)"
                                                </span>
                                            }.into_any()
                                        }}
                                    </div>
                                </header>

                                // If locked, display the Locked Hero Card with Purchase Callout:
                                {if !is_unlocked {
                                    view! {
                                        <div class="locked-course-card">
                                            <div class="locked-course-layout">
                                                <div class="locked-course-icon">"🔒"</div>
                                                <div class="locked-course-copy">
                                                    <h3 class="locked-course-title">
                                                        "本课程讲义已锁定 (Course Locked)"
                                                    </h3>
                                                    <p class="locked-course-description">
                                                        {format!("购买整门课程即可解锁其所有讲义，共包含 {} 页精讲课件、历史背景深度剖析及多媒体内容。未购买学员可以免费预览第 1 页。", deck.slide_count)}
                                                    </p>
                                                    {if status.is_logged_in {
                                                        let uname = status.username.clone().unwrap_or_default();
                                                        view! {
                                                            <div class="locked-account-row">
                                                                <div class="locked-account-details">
                                                                    <div class="locked-account-name">
                                                                        {format!("当前学员账户：{}", uname)}
                                                                    </div>
                                                                    <div class="locked-account-price">
                                                                        "按课程购买；本原型暂不处理实际付款。"
                                                                    </div>
                                                                </div>
                                                                <div>
                                                                    <ActionForm action=buy_action>
                                                                        <input type="hidden" name="deck_id" value=deck.id/>
                                                                        <input type="hidden" name="learner_id" value=learner_id/>
                                                                        <button
                                                                            type="submit"
                                                                            class="purchase-button"
                                                                        >
                                                                            "解锁整门课程"
                                                                        </button>
                                                                    </ActionForm>
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <div class="locked-account-row">
                                                                <div class="locked-account-details">
                                                                    <div class="locked-account-name">
                                                                        "您当前尚未登录账户"
                                                                    </div>
                                                                    <div class="locked-account-message">
                                                                        "登录学员账户后即可一键购买或查看已获权限。"
                                                                    </div>
                                                                </div>
                                                                <div>
                                                                    <A href="/login" attr:class="button login-button">
                                                                        "🔑 登录 / 注册账户 →"
                                                                    </A>
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    }}
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    ().into_any()
                                }}

                                // Search bar
                                <div class="search-wrap">
                                    <input
                                        type="text"
                                        class="search-box"
                                        placeholder=if is_unlocked {
                                            "🔍 快速搜索本讲内容（输入关键词，实时过滤）..."
                                        } else {
                                            "🔍 搜索课程大纲（购买解锁后可全文检索全部讲义与正文）..."
                                        }
                                        on:input=move |ev| set_query.set(event_target_value(&ev))
                                        prop:value=query
                                    />
                                </div>

                                <div class="deck-content-layout">
                                    <aside class="deck-inventory" id="toc" aria-label="Slide inventory">
                                        <div class="deck-inventory-heading">
                                            <div>
                                                <p>"DECK GUIDE"</p>
                                                <h2>"Slide inventory"</h2>
                                            </div>
                                            <span>{deck.slide_count}</span>
                                        </div>
                                        {if !is_unlocked {
                                            view! {
                                                <p class="deck-inventory-note">"Slide 1 is the free preview. The remaining slides are locked."</p>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <p class="deck-inventory-note">"Select a slide to jump to it."</p>
                                            }.into_any()
                                        }}
                                        <nav aria-label="Slides in this deck">
                                            <ul class="deck-inventory-list">
                                                {move || {
                                                    let q = query.get().trim().to_lowercase();
                                                    deck.slides.iter().filter(|slide| {
                                                        q.is_empty()
                                                            || slide.title.to_lowercase().contains(&q)
                                                            || (is_unlocked && slide.paragraphs.iter().any(|paragraph| paragraph.to_lowercase().contains(&q)))
                                                            || format!("{}", slide.num).contains(&q)
                                                    }).map(|slide| {
                                                        let num = slide.num;
                                                        let is_slide_locked = !is_unlocked && num > 1;
                                                        let has_video = slide.media.iter().any(|media| media.media_type == MediaType::Video);
                                                        let anchor = format!("#slide-{num}");
                                                        let progress_action = progress_action.clone();
                                                        let progress_deck_id = deck.id.to_string();
                                                        view! {
                                                            <li>
                                                                <a class="deck-inventory-link" href=anchor on:click=move |_| {
                                                                    if is_unlocked {
                                                                        progress_action.dispatch(RecordCourseProgress {
                                                                            learner_id,
                                                                            course_id,
                                                                            deck_id: progress_deck_id.clone(),
                                                                            slide_num: num as i64,
                                                                        });
                                                                    }
                                                                }>
                                                                    <span class="deck-inventory-number">{num}</span>
                                                                    <span class="deck-inventory-title">{slide.title}</span>
                                                                    {if is_slide_locked {
                                                                        view! { <span class="deck-inventory-lock" title="需购买解锁">"🔒"</span> }.into_any()
                                                                    } else if !is_unlocked {
                                                                        view! { <span class="deck-inventory-preview">"PREVIEW"</span> }.into_any()
                                                                    } else {
                                                                        ().into_any()
                                                                    }}
                                                                    {if has_video {
                                                                        view! { <span class="deck-inventory-media" title="Includes video">"▶"</span> }.into_any()
                                                                    } else {
                                                                        ().into_any()
                                                                    }}
                                                                </a>
                                                            </li>
                                                        }
                                                    }).collect_view()
                                                }}
                                            </ul>
                                        </nav>
                                    </aside>

                                    // Slide Items
                                    <div id="lecture-list" class="lecture-list">
                                    {move || {
                                        let q = query.get().trim().to_lowercase();
                                        let slides = deck.slides;
                                        let filtered: Vec<&Slide> = if q.is_empty() {
                                            slides.iter().collect()
                                        } else {
                                            slides.iter().filter(|s| {
                                                s.title.to_lowercase().contains(&q)
                                                    || (is_unlocked && s.paragraphs.iter().any(|p| p.to_lowercase().contains(&q)))
                                                    || format!("{}", s.num).contains(&q)
                                            }).collect()
                                        };

                                        if filtered.is_empty() {
                                            view! {
                                                <div class="empty-search-results">
                                                    "🔍 没有找到与关键词匹配的讲义大纲。"
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div>
                                                    {filtered.into_iter().map(|s| {
                                                        let is_slide_locked = !is_unlocked && s.num > 1;
                                                        let is_preview = !is_unlocked && s.num == 1;
                                                        render_slide(deck.id, s, is_slide_locked, is_preview)
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                    </div>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </Transition>
        </div>
    }
}

fn render_slide(deck_id: &'static str, s: &Slide, is_locked: bool, is_preview: bool) -> impl IntoView {
    let num = s.num;
    let title = s.title;
    let slide_id = format!("slide-{}", num);

    view! {
        <article
            class="lecture-item"
            id=slide_id
        >
            <div class="item-header">
                {if is_locked {
                    view! {
                        <span class="slide-badge slide-badge--locked">
                            {format!("第 {} 页 🔒", num)}
                        </span>
                    }.into_any()
                } else if is_preview {
                    view! {
                        <span class="slide-badge slide-badge--preview">
                            {format!("第 {} 页 (试读)", num)}
                        </span>
                    }.into_any()
                } else {
                    view! {
                        <span class="slide-badge slide-badge--standard">
                            {format!("第 {} 页", num)}
                        </span>
                    }.into_any()
                }}

                <h2 class="slide-title">
                    {title}
                </h2>
                <a href="" class="top-link">"↑ 目录"</a>
            </div>

            {if is_locked {
                // Locked Slide Placeholder:
                view! {
                    <div class="locked-slide-placeholder">
                        <div class="locked-slide-icon">"🔒"</div>
                        <div class="locked-slide-title">
                            "本页讲义内容已锁定 (付费学员专享)"
                        </div>
                        <p class="locked-slide-copy">
                            "本页包含该章节完整图文讲解、核心考点提炼及随堂嵌入式音视频媒体。购买解锁本课程后即可畅读全部讲义内容。"
                        </p>
                        <a
                            href="#toc"
                            class="locked-slide-link"
                        >
                            "查看课程购买 / 解锁选项 ↑"
                        </a>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div>
                        // Media
                        {s.media.iter().map(|m| {
                            let src_url = format!("/slides-media/{}/{}", deck_id, m.src);
                            match m.media_type {
                                MediaType::Video => {
                                    let poster_url = m.poster.map(|p| format!("/slides-media/{}/{}", deck_id, p));
                                    view! {
                                        <div class="lecture-media video-box">
                                            <video
                                                controls
                                                poster=poster_url
                                                preload="metadata"
                                                class="lecture-video"
                                            >
                                                <source src=src_url type="video/mp4"/>
                                                "您的浏览器不支持直接播放此视频。"
                                            </video>
                                            <div class="media-caption">
                                                {format!("▶ 视频资料（第 {} 页）", num)}
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                                MediaType::Image => {
                                    let caption = m.caption.unwrap_or("");
                                    view! {
                                        <div class="lecture-media image-box">
                                            <img
                                                src=src_url
                                                alt=caption
                                                loading="lazy"
                                                class="lecture-image"
                                            />
                                        </div>
                                    }.into_any()
                                }
                            }
                        }).collect_view()}

                        // Paragraphs
                        <div class="item-content">
                            {if s.paragraphs.is_empty() {
                                view! {
                                    <p class="empty-note">
                                        "（本页为图档/视频内容）"
                                    </p>
                                }.into_any()
                            } else {
                                view! {
                                    <div>
                                        {s.paragraphs.iter().map(|p| {
                                            view! {
                                                <p class="slide-paragraph">
                                                    {*p}
                                                </p>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    </div>
                }.into_any()
            }}
        </article>
    }
}
