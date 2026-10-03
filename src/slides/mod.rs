// ============================================================
//  slides/mod.rs — Leptos SSR Slide Decks & Lecture Reader
// ============================================================

pub mod access;
pub mod model;
pub mod israel_palestine_1;
pub mod israel_palestine_2;

pub use access::*;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{components::A, hooks::use_params_map};
pub use model::{MediaItem, MediaType, Slide, SlideDeck};

pub const ALL_DECKS: &[SlideDeck] = &[
    israel_palestine_1::DECK,
    israel_palestine_2::DECK,
];

pub fn find_deck(id: &str) -> Option<SlideDeck> {
    ALL_DECKS.iter().copied().find(|d| d.id == id)
}

/// Dynamic Route component for `/slides/:id`
#[component]
pub fn SlideDeckPage() -> impl IntoView {
    let params = use_params_map();
    let deck_id = move || params.read().get("id").unwrap_or_default();

    view! {
        {move || {
            let id = deck_id();
            match find_deck(&id) {
                Some(deck) => view! {
                    <SlideDeckViewer deck=deck />
                }.into_any(),
                None => view! {
                    <div class="slides-not-found" style="max-width: 600px; margin: 40px auto; text-align: center; padding: 30px; background: #ffffff; border-radius: 8px; border: 1px solid #e7e2d7;">
                        <h2>"未找到该讲义"</h2>
                        <p style="color: #6b7280; margin: 12px 0 20px 0;">
                            {format!("不存在 ID 为 \"{}\" 的讲义速读。", id)}
                        </p>
                        <A href="/slides" attr:class="button">"← 返回讲义速读列表"</A>
                    </div>
                }.into_any(),
            }
        }}
    }
}

/// Index page for `/slides` listing all available lecture slide decks.
#[component]
pub fn SlidesIndexPage() -> impl IntoView {
    let unlocked = Resource::new(|| (), |_| async { get_unlocked_courses().await.unwrap_or_default() });

    view! {
        <Title text="课程讲义速读 — 课件与速读大纲"/>
        <div class="slides-index-page" style="max-width: 860px; margin: 0 auto; padding: 24px 16px 60px 16px;">
            <header style="margin-bottom: 30px; padding-bottom: 20px; border-bottom: 2px solid #e7e2d7; background: transparent; color: inherit; text-align: left; display: block;">
                <div style="margin-bottom: 12px;">
                    <A href="/materials" attr:style="color: #6b7280; text-decoration: none; font-size: 14px;">"← 返回课程资料库"</A>
                </div>
                <h1 style="font-size: 28px; font-weight: 700; color: #1e293b; margin: 0 0 8px 0;">
                    "📖 课程讲义速读 (Slide Lecture Notes)"
                </h1>
                <p style="margin: 0; color: #6b7280; font-size: 15px;">
                    "配套 PPT 讲义精炼速读版与多媒体内容，支持全文检索与分章节复习。完全基于 Leptos SSR 渲染，流畅原生体验。"
                </p>
            </header>

            <Transition fallback=move || view! { <p class="loading">"正在加载课程列表..."</p> }>
                {move || match unlocked.get() {
                    None => ().into_any(),
                    Some(unlocked_list) => {
                        view! {
                            <div class="deck-grid" style="display: grid; gap: 20px;">
                                {ALL_DECKS.iter().map(|deck| {
                                    let deck_id = deck.id;
                                    let deck_title = deck.title;
                                    let slide_count = deck.slide_count;
                                    let href = format!("/slides/{}", deck_id);
                                    let is_unlocked = unlocked_list.contains(&deck_id.to_string());

                                    view! {
                                        <div class="deck-card" style="background: #ffffff; border: 1px solid #e7e2d7; border-radius: 10px; padding: 20px; display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap; box-shadow: 0 1px 4px rgba(0,0,0,0.03);">
                                            <div>
                                                <div style="font-weight: 700; font-size: 1.2rem; color: #29251f; margin-bottom: 6px; display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                                                    <span>{deck_title}</span>
                                                    {if is_unlocked {
                                                        view! {
                                                            <span style="background: #dcfce7; color: #15803d; font-size: 0.78rem; font-weight: 600; padding: 2px 8px; border-radius: 4px;">
                                                                "已解锁"
                                                            </span>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <span style="background: #fef3c7; color: #b45309; font-size: 0.78rem; font-weight: 600; padding: 2px 8px; border-radius: 4px;">
                                                                "🔒 需购买 · ￥29.9"
                                                            </span>
                                                        }.into_any()
                                                    }}
                                                </div>
                                                <div style="font-size: 0.88rem; color: #7a6e60; display: flex; gap: 10px; align-items: center; flex-wrap: wrap;">
                                                    <span style="background: #eadecd; padding: 2px 8px; border-radius: 4px; font-weight: 600; color: #4b3e2e;">
                                                        {format!("{} 页讲义", slide_count)}
                                                    </span>
                                                    <span>{format!("ID: {}", deck_id)}</span>
                                                    <span>"• 含多媒体视频/图片"</span>
                                                </div>
                                            </div>
                                            <div>
                                                <A
                                                    href=href
                                                    attr:class="button"
                                                    attr:style=if is_unlocked {
                                                        "background: #1d4ed8; color: #ffffff; padding: 8px 18px; border-radius: 6px; text-decoration: none; font-weight: 600; display: inline-block;"
                                                    } else {
                                                        "background: #d97706; color: #ffffff; padding: 8px 18px; border-radius: 6px; text-decoration: none; font-weight: 600; display: inline-block;"
                                                    }
                                                >
                                                    {if is_unlocked { "开始学习 →" } else { "查看讲义 (试读/购买) →" }}
                                                </A>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }}
            </Transition>
        </div>
    }
}

/// The core Leptos SSR Slide Viewer Component with Access Control & Locked Preview
#[component]
pub fn SlideDeckViewer(deck: SlideDeck) -> impl IntoView {
    let deck_id = deck.id.to_string();
    let access = Resource::new(
        move || deck_id.clone(),
        |id| async move { check_course_access(id).await.unwrap_or_default() },
    );
    let buy_action = ServerAction::<BuyCourseAccess>::new();

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
        <div class="slide-deck-viewer" style="max-width: 860px; margin: 0 auto; padding: 24px 16px 60px 16px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', Georgia, serif; line-height: 1.8; color: #29251f;">
            <Transition fallback=move || view! { <p class="loading">"正在加载讲义与权限..."</p> }>
                {move || match access.get() {
                    None => ().into_any(),
                    Some(status) => {
                        let is_unlocked = status.is_unlocked;

                        view! {
                            <div>
                                <header class="deck-header" style="margin-bottom: 24px; padding-bottom: 18px; border-bottom: 2px solid #e7e2d7; background: transparent; color: inherit; text-align: left; display: block;">
                                    <div class="back-nav" style="margin-bottom: 12px; font-size: 14px;">
                                        <A href="/materials" attr:style="color: #6b7280; text-decoration: none;">"← 课程资料库"</A>
                                        <span style="color: #cbd5e1; margin: 0 8px;">"|"</span>
                                        <A href="/slides" attr:style="color: #6b7280; text-decoration: none;">"全部讲义"</A>
                                    </div>
                                    <h1 style="font-size: 28px; font-weight: 700; margin: 0 0 10px 0; color: #1e293b;">
                                        {deck.title}
                                    </h1>
                                    <div class="deck-meta" style="font-size: 14px; color: #6b7280; display: flex; gap: 14px; flex-wrap: wrap; align-items: center;">
                                        <span>{format!("📑 共 {} 页讲义", deck.slide_count)}</span>
                                        <span>"🎬 嵌入式视频/图档"</span>
                                        {if is_unlocked {
                                            if status.is_admin {
                                                view! { <span style="background: #e0f2fe; color: #0369a1; padding: 2px 8px; border-radius: 4px; font-weight: 600; font-size: 13px;">"👑 管理员授权"</span> }.into_any()
                                            } else {
                                                view! { <span style="background: #dcfce7; color: #15803d; padding: 2px 8px; border-radius: 4px; font-weight: 600; font-size: 13px;">"✅ 已解锁完整讲义"</span> }.into_any()
                                            }
                                        } else {
                                            view! {
                                                <span style="background: #fef3c7; color: #b45309; padding: 2px 8px; border-radius: 4px; font-weight: 600; font-size: 13px;">
                                                    "🔒 未解锁 (试读模式)"
                                                </span>
                                            }.into_any()
                                        }}
                                    </div>
                                </header>

                                // If locked, display the Locked Hero Card with Purchase Callout:
                                {if !is_unlocked {
                                    view! {
                                        <div class="locked-course-card" style="background: #fffdf5; border: 2px solid #f59e0b; border-radius: 12px; padding: 22px 24px; margin-bottom: 28px; box-shadow: 0 4px 12px rgba(245, 158, 11, 0.1);">
                                            <div style="display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap;">
                                                <div style="font-size: 34px; line-height: 1;">"🔒"</div>
                                                <div style="flex: 1; min-width: 260px;">
                                                    <h3 style="margin: 0 0 6px 0; font-size: 19px; color: #92400e; font-weight: 700;">
                                                        "本课程讲义已锁定 (Course Locked)"
                                                    </h3>
                                                    <p style="margin: 0 0 14px 0; color: #78350f; font-size: 14.5px; line-height: 1.6;">
                                                        {format!("当前讲义共包含 {} 页精讲课件、历史背景深度剖析及随堂原版多媒体视频。未购买学员仅可免费浏览课程目录大纲与第 1 页试读内容。", deck.slide_count)}
                                                    </p>
                                                    {if status.is_logged_in {
                                                        let uname = status.username.clone().unwrap_or_default();
                                                        view! {
                                                            <div style="background: #ffffff; border: 1px solid #fde68a; border-radius: 8px; padding: 14px 18px; display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap;">
                                                                <div>
                                                                    <div style="font-weight: 600; color: #29251f; font-size: 14px;">
                                                                        {format!("当前学员账户：{}", uname)}
                                                                    </div>
                                                                    <div style="font-size: 14px; color: #b45309; font-weight: 700; margin-top: 3px;">
                                                                        "课程特惠价：￥29.9 / 终身永久阅读"
                                                                    </div>
                                                                </div>
                                                                <div>
                                                                    <ActionForm action=buy_action>
                                                                        <input type="hidden" name="deck_id" value=deck.id/>
                                                                        <button
                                                                            type="submit"
                                                                            style="background: #d97706; color: #ffffff; border: none; padding: 10px 22px; border-radius: 6px; font-weight: 700; font-size: 14.5px; cursor: pointer; box-shadow: 0 2px 6px rgba(217, 119, 6, 0.3);"
                                                                        >
                                                                            "💳 立即购买 / 解锁课程"
                                                                        </button>
                                                                    </ActionForm>
                                                                </div>
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <div style="background: #ffffff; border: 1px solid #fde68a; border-radius: 8px; padding: 14px 18px; display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap;">
                                                                <div>
                                                                    <div style="font-weight: 600; color: #29251f; font-size: 14px;">
                                                                        "您当前尚未登录账户"
                                                                    </div>
                                                                    <div style="font-size: 13px; color: #78350f; margin-top: 2px;">
                                                                        "登录学员账户后即可一键购买或查看已获权限。"
                                                                    </div>
                                                                </div>
                                                                <div>
                                                                    <A href="/login" attr:class="button" attr:style="background: #d97706; color: #ffffff; padding: 8px 18px; border-radius: 6px; text-decoration: none; font-weight: 600; font-size: 14px; display: inline-block;">
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
                                <div class="search-wrap" style="margin: 20px 0; position: sticky; top: 12px; z-index: 50; padding: 6px 0; background: #f5f1e8;">
                                    <input
                                        type="text"
                                        class="search-box"
                                        placeholder=if is_unlocked {
                                            "🔍 快速搜索本讲内容（输入关键词，实时过滤）..."
                                        } else {
                                            "🔍 搜索课程大纲（购买解锁后可全文检索全部讲义与正文）..."
                                        }
                                        style="width: 100%; padding: 12px 16px; font-size: 15px; border: 1px solid #cbd5e1; border-radius: 8px; background: #ffffff; box-shadow: 0 2px 6px rgba(0,0,0,0.04); outline: none; font-family: inherit;"
                                        on:input=move |ev| set_query.set(event_target_value(&ev))
                                        prop:value=query
                                    />
                                </div>

                                // Table of Contents
                                <details class="toc-card" open style="background: #ffffff; border: 1px solid #e7e2d7; border-radius: 8px; padding: 14px 18px; margin-bottom: 30px;">
                                    <summary style="font-weight: 600; cursor: pointer; color: #1d4ed8; user-select: none; display: flex; justify-content: space-between; align-items: center;">
                                        <span>"📑 目录大纲 (点击展开/收起)"</span>
                                        {if !is_unlocked {
                                            view! {
                                                <span style="font-size: 12px; color: #b45309; font-weight: normal;">
                                                    "第 1 页试读 · 第 2-90 页已锁定 🔒"
                                                </span>
                                            }.into_any()
                                        } else {
                                            ().into_any()
                                        }}
                                    </summary>
                                    <ul class="toc-list" style="list-style: none; margin: 12px 0 0 0; padding: 0 0 0 4px; max-height: 280px; overflow-y: auto;">
                                        {deck.slides.iter().map(|s| {
                                            let num = s.num;
                                            let t = s.title;
                                            let has_video = s.media.iter().any(|m| m.media_type == MediaType::Video);
                                            let anchor = format!("#slide-{}", num);

                                            view! {
                                                <li style="padding: 4px 0; font-size: 14px;">
                                                    <a href=anchor style="color: #29251f; text-decoration: none; display: flex; align-items: center; gap: 6px;">
                                                        <span style="color: #6b7280; font-size: 12px; min-width: 24px;">{format!("{}.", num)}</span>
                                                        <span style="flex: 1;">{t}</span>
                                                        {if !is_unlocked {
                                                            if num == 1 {
                                                                view! {
                                                                    <span style="background: #e0f2fe; color: #0284c7; padding: 1px 6px; border-radius: 3px; font-size: 11px; font-weight: 600;">
                                                                        "试读"
                                                                    </span>
                                                                }.into_any()
                                                            } else {
                                                                view! {
                                                                    <span style="color: #d97706; font-size: 12px;" title="需购买解锁">
                                                                        "🔒"
                                                                    </span>
                                                                }.into_any()
                                                            }
                                                        } else {
                                                            ().into_any()
                                                        }}
                                                        {if has_video {
                                                            view! { <span style="margin-left: 4px;">"🎬"</span> }.into_any()
                                                        } else {
                                                            ().into_any()
                                                        }}
                                                    </a>
                                                </li>
                                            }
                                        }).collect_view()}
                                    </ul>
                                </details>

                                // Slide Items
                                <main id="lecture-list">
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
                                                <div style="text-align: center; padding: 40px; color: #6b7280; background: #ffffff; border-radius: 8px; border: 1px dashed #cbd5e1;">
                                                    "🔍 没有找到与关键词匹配的讲义大纲。"
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div>
                                                    {filtered.into_iter().map(|s| {
                                                        let is_slide_locked = !is_unlocked && s.num > 1;
                                                        render_slide(deck.id, s, is_slide_locked)
                                                    }).collect_view()}
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </main>
                            </div>
                        }.into_any()
                    }
                }}
            </Transition>
        </div>
    }
}

fn render_slide(deck_id: &'static str, s: &Slide, is_locked: bool) -> impl IntoView {
    let num = s.num;
    let title = s.title;
    let slide_id = format!("slide-{}", num);

    view! {
        <article
            class="lecture-item"
            id=slide_id
            style="background: #ffffff; border: 1px solid #e7e2d7; border-radius: 10px; padding: 24px; margin-bottom: 24px; box-shadow: 0 1px 4px rgba(0,0,0,0.03); scroll-margin-top: 80px;"
        >
            <div class="item-header" style="display: flex; align-items: baseline; gap: 12px; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px dashed #e7e2d7;">
                {if is_locked {
                    view! {
                        <span class="slide-badge" style="font-size: 12px; font-weight: 700; color: #92400e; background: #fef3c7; padding: 2px 8px; border-radius: 4px; flex-shrink: 0;">
                            {format!("第 {} 页 🔒", num)}
                        </span>
                    }.into_any()
                } else if num == 1 {
                    view! {
                        <span class="slide-badge" style="font-size: 12px; font-weight: 700; color: #0369a1; background: #e0f2fe; padding: 2px 8px; border-radius: 4px; flex-shrink: 0;">
                            {format!("第 {} 页 (试读)", num)}
                        </span>
                    }.into_any()
                } else {
                    view! {
                        <span class="slide-badge" style="font-size: 12px; font-weight: 700; color: #4b5563; background: #f1f5f9; padding: 2px 8px; border-radius: 4px; flex-shrink: 0;">
                            {format!("第 {} 页", num)}
                        </span>
                    }.into_any()
                }}

                <h2 style="font-size: 20px; font-weight: 700; flex: 1; color: #1e3a8a; line-height: 1.4; margin: 0;">
                    {title}
                </h2>
                <a href="#toc" class="top-link" style="font-size: 12px; color: #6b7280; text-decoration: none;">"↑ 目录"</a>
            </div>

            {if is_locked {
                // Locked Slide Placeholder:
                view! {
                    <div class="locked-slide-placeholder" style="background: #fafaf9; border: 1px dashed #cbd5e1; border-radius: 8px; padding: 32px 20px; text-align: center; margin: 16px 0;">
                        <div style="font-size: 32px; margin-bottom: 8px;">"🔒"</div>
                        <div style="font-weight: 700; color: #44403c; font-size: 16px; margin-bottom: 6px;">
                            "本页讲义内容已锁定 (付费学员专享)"
                        </div>
                        <p style="font-size: 13.5px; color: #78716c; max-width: 500px; margin: 0 auto 16px auto; line-height: 1.6;">
                            "本页包含该章节完整图文讲解、核心考点提炼及随堂嵌入式音视频媒体。购买解锁本课程后即可畅读全部讲义内容。"
                        </p>
                        <a
                            href="#toc"
                            style="display: inline-block; background: #d97706; color: #ffffff; padding: 8px 18px; border-radius: 6px; font-size: 13.5px; font-weight: 600; text-decoration: none; box-shadow: 0 2px 4px rgba(217, 119, 6, 0.2);"
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
                                        <div class="lecture-media video-box" style="margin: 16px 0; border-radius: 6px; overflow: hidden; background: #000000; display: flex; flex-direction: column; align-items: center;">
                                            <video
                                                controls
                                                poster=poster_url
                                                preload="metadata"
                                                style="width: 100%; max-height: 480px; outline: none;"
                                            >
                                                <source src=src_url type="video/mp4"/>
                                                "您的浏览器不支持直接播放此视频。"
                                            </video>
                                            <div class="media-caption" style="background: #1e293b; color: #e2e8f0; font-size: 12px; padding: 6px 12px; width: 100%; text-align: center;">
                                                {format!("▶ 视频资料（第 {} 页）", num)}
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                                MediaType::Image => {
                                    let caption = m.caption.unwrap_or("");
                                    view! {
                                        <div class="lecture-media image-box" style="margin: 16px 0; border-radius: 6px; overflow: hidden; background: #000000; display: flex; flex-direction: column; align-items: center;">
                                            <img
                                                src=src_url
                                                alt=caption
                                                loading="lazy"
                                                style="max-width: 100%; height: auto; max-height: 480px; display: block; margin: 0 auto;"
                                            />
                                        </div>
                                    }.into_any()
                                }
                            }
                        }).collect_view()}

                        // Paragraphs
                        <div class="item-content" style="font-size: 16px; line-height: 1.8;">
                            {if s.paragraphs.is_empty() {
                                view! {
                                    <p class="empty-note" style="color: #6b7280; font-style: italic; font-size: 14px; margin: 0;">
                                        "（本页为图档/视频内容）"
                                    </p>
                                }.into_any()
                            } else {
                                view! {
                                    <div>
                                        {s.paragraphs.iter().map(|p| {
                                            view! {
                                                <p style="margin-bottom: 12px; text-align: justify;">
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
