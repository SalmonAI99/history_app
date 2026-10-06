// Public home and teacher pages, blog/news listings and details, and the admin page.

use std::time::Duration;

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use crate::posts::{get_post, get_posts};

mod teacher;
pub use teacher::TeacherPage;

// ----------------------------------------------------------
//  HOME — the marketing front door.
// ----------------------------------------------------------

// Headlines the hero types out, one after another.
const HERO_LINES: [&str; 3] = ["读懂历史，看懂世界", "培养习惯，终身自学", "启发思考，理解现实"];

#[component]
pub fn HomePage() -> impl IntoView {
    // The server renders the first line in full; once hydrated, the browser
    // starts erasing and typing through HERO_LINES.
    let headline = RwSignal::new(HERO_LINES[0].to_string());
    Effect::new(move |_| {
        let len = HERO_LINES[0].chars().count();
        set_timeout(move || type_step(headline, 0, len, true), Duration::from_millis(2000));
    });

    view! {
        <Title text="星阅读 StarRead — 儿童阅读课程"/>


        <div class="home">
            <section class="home-hero">
                <div class="home-hero-content">
                    <p class="home-eyebrow">"HISTORY · READING · THINKING"</p>
                    <h1 class="home-hero-headline" aria-label=HERO_LINES[0]>
                        <span aria-hidden="true">{move || headline.get()}</span>
                        <span class="home-hero-cursor" aria-hidden="true"></span>
                    </h1>
                    <p class="home-hero-subline">
                        "通过精心编排的故事、动态地图与视频，带孩子走进历史现场，在沉浸式学习中启发独立思考。"
                    </p>
                    <div class="home-hero-ctas">
                        <A href="/classes" attr:class="home-btn home-btn--primary">"浏览课程 →"</A>
                        <A href="/teacher" attr:class="home-btn home-btn--secondary">"认识老师"</A>
                    </div>
                    <div class="home-hero-proof">
                        <span>"2 门核心课程"</span>
                        <span aria-hidden="true">"·"</span>
                        <span>"90+ 张精美插画课件"</span>
                        <span aria-hidden="true">"·"</span>
                        <span>"20+ 段深度解析视频"</span>
                    </div>
                </div>

                <div class="home-hero-media">
                    <div class="home-hero-frame"></div>
                    <video autoplay muted loop playsinline preload="auto">
                        <source src="/editorial_MASTER_1791176464742.mp4" type="video/mp4"/>
                    </video>
                </div>
            </section>

            // ---------- Core abilities ----------
            <section class="home-section">
                <div class="home-container">
                    <div class="home-section-header">
                        <h2 class="home-section-title">"培养核心能力"</h2>
                        <p class="home-section-desc">"给孩子受用一生的认知工具，帮助他们建立理解世界的框架。"</p>
                    </div>
                    <div class="home-abilities">
                        <AbilityCard title="建立世界观"
                            text="在早期接触宏大的历史脉络，帮助孩子尽早建立对世界的全局认知与宏观视角。">
                            <circle cx="12" cy="12" r="10"/>
                            <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/>
                            <path d="M2 12h20"/>
                        </AbilityCard>
                        <AbilityCard title="理解真实世界"
                            text="通过历史事件的因果分析与底层逻辑，培养孩子洞察社会现象和理解现实问题的能力。">
                            <circle cx="12" cy="12" r="10"/>
                            <path d="m16.24 7.76-2.12 6.36-6.36 2.12 2.12-6.36 6.36-2.12z"/>
                        </AbilityCard>
                        <AbilityCard title="培养阅读习惯"
                            text="以生动的故事激发兴趣，引导孩子从被动听课自然过渡到主动阅读长篇严肃文本。">
                            <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/>
                            <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
                        </AbilityCard>
                        <AbilityCard title="终身自学能力"
                            text="授人以渔，掌握收集信息、交叉比对和独立思考的史学方法论，培养自驱力。">
                            <path d="M15 14c.2-1 .7-1.7 1.5-2.5 1-.9 1.5-2.2 1.5-3.5A6 6 0 0 0 6 8c0 1 .2 2.2 1.5 3.5.7.7 1.3 1.5 1.5 2.5"/>
                            <path d="M9 18h6"/>
                            <path d="M10 22h4"/>
                        </AbilityCard>
                    </div>
                    <p class="home-abilities-quote">
                        "“历史课程只是基石，独立思考与终身学习才是最终的目的。”"
                    </p>
                </div>
            </section>

            // ---------- Teaching methods ----------
            <section class="home-section home-methods">
                <div class="home-container">
                    <div class="home-section-header">
                        <h2 class="home-section-title">"沉浸式教学方法"</h2>
                        <p class="home-section-desc">"打破枯燥的年代背诵，用现代媒介还原历史现场"</p>
                    </div>

                    <div class="home-method">
                        <div class="home-method-content">
                            <h3>"多媒体辅助教学"</h3>
                            <p>"结合精心设计的幻灯片、高质量纪录片片段与动态地图。每节课都像是一场视觉丰富的博物馆导览，让抽象的历史概念变得触手可及，极大提高学生的专注力与理解深度。"</p>
                        </div>
                        <figure class="home-method-visual">
                            <img src="/images/Fixing_toilet_in_ancient_times_20261004222952.jpg"
                                alt="课件插画：古代生活场景" loading="lazy"/>
                        </figure>
                    </div>

                    <div class="home-method">
                        <div class="home-method-content">
                            <h3>"3D 沉浸式历史场景"</h3>
                            <p>"利用 3D 模拟器还原古代城市布局、经典战役地形与伟大建筑的建造过程。给孩子一种“身临其境”的历史涌现感，加深空间和时间记忆，在脑海中构建鲜活的历史图景。"</p>
                        </div>
                        <figure class="home-method-visual">
                            <img src="/images/middle-east-course.jpg"
                                alt="3D 历史沙盘：古代两河流域城市" loading="lazy"/>
                        </figure>
                    </div>
                </div>
            </section>

            // ---------- Reviews ----------
            <section class="home-section">
                <div class="home-container">
                    <div class="home-section-header">
                        <h2 class="home-section-title">"家长与学生反馈"</h2>
                    </div>
                    <ReviewSlider/>
                </div>
            </section>

            // ---------- Contact & links ----------
            <section class="home-section home-contact">
                <div class="home-container home-contact-grid">
                    <div class="home-contact-info">
                        <h2>"开启历史探索之旅"</h2>
                        <p>"我们提供适合不同年龄段的在线历史启蒙与进阶课程，帮助孩子在阅读与思考中成长。有任何疑问，欢迎随时与我们交流。"</p>
                        <a href="mailto:hello@teacherwang.com" class="home-btn home-btn--primary">"预约咨询"</a>
                    </div>
                    <div class="home-contact-links">
                        <div>
                            <h3>"探索资源"</h3>
                            <ul>
                                <li><A href="/classes">"全部课程"</A></li>
                                <li><A href="/blog">"历史阅读文章"</A></li>
                                <li><A href="/news">"最新活动"</A></li>
                            </ul>
                        </div>
                        <div>
                            <h3>"联系我们"</h3>
                            <ul>
                                <li><a href="mailto:hello@teacherwang.com">"hello@teacherwang.com"</a></li>
                                <li>"微信公众号：TeacherWang"</li>
                            </ul>
                        </div>
                    </div>
                </div>
            </section>

            <SiteFooter/>
        </div>
    }
}

/// Shared bottom navigation used by the home page and public information pages.
#[component]
pub(super) fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="site-footer">
            <nav class="site-footer-links" aria-label="页脚导航">
                <A href="/classes">"课程中心"</A>
                <A href="/teacher">"师资介绍"</A>
                <A href="/blog">"文章"</A>
                <A href="/news">"活动"</A>
            </nav>
            <p class="site-copyright">"© 2026 王老师历史课。保留所有权利。"</p>
        </footer>
    }
}

/// One tick of the hero typewriter: add or remove a character, then
/// schedule the next tick. Stops once the page is gone (signal disposed).
fn type_step(text: RwSignal<String>, line: usize, shown: usize, deleting: bool) {
    let chars: Vec<char> = HERO_LINES[line].chars().collect();
    let shown = if deleting { shown - 1 } else { shown + 1 };
    if text.try_set(chars[..shown].iter().collect()).is_some() {
        return;
    }

    let (mut line, mut deleting) = (line, deleting);
    let mut delay = if deleting { 40 } else { 120 };
    if !deleting && chars[shown - 1] == '，' {
        delay = 400; // pause at the comma
    }
    if !deleting && shown == chars.len() {
        delay = 2000;
        deleting = true;
    } else if deleting && shown == 0 {
        deleting = false;
        line = (line + 1) % HERO_LINES.len();
        delay = 400;
    }

    set_timeout(move || type_step(text, line, shown, deleting), Duration::from_millis(delay));
}

/// A feature card. `children` are the SVG shapes of a 24×24 line icon.
#[component]
fn AbilityCard(title: &'static str, text: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="home-ability">
            <div class="home-ability-icon">
                <svg viewBox="0 0 24 24" aria-hidden="true">{children()}</svg>
            </div>
            <h3>{title}</h3>
            <p>{text}</p>
        </div>
    }
}

// TODO: placeholder testimonials from the design mock — replace with real ones.
// (who, where, what they said) — add as many as you like; the slider scrolls.
const REVIEWS: &[(&str, &str, &str)] = &[
    ("六年级学生家长", "上海",
     "自从上了王老师的课，孩子开始主动看家里买的那些大部头历史书了。课件做得非常精美，像在看一部优秀的纪录片，连我们做家长的都跟着一起听得津津有味。"),
    ("初二学生", "北京",
     "以前觉得历史就是背年份，现在我明白了那些事件为什么会发生。3D地图那节课太震撼了，感觉自己就在古罗马的街道上走着，历史课成了我每周最期待的事情。"),
];

/// Horizontally scrolling, snap-aligned review cards with prev/next arrows.
/// The arrows hide themselves when every card already fits.
#[component]
fn ReviewSlider() -> impl IntoView {
    let track = NodeRef::<leptos::html::Div>::new();
    let at_start = RwSignal::new(true);
    let at_end = RwSignal::new(false);

    let update = move || {
        if let Some(el) = track.get_untracked() {
            let left = el.scroll_left();
            at_start.set(left <= 1);
            at_end.set(left + el.client_width() >= el.scroll_width() - 1);
        }
    };
    let scroll = move |dir: f64| {
        if let Some(el) = track.get_untracked() {
            el.scroll_by_with_x_and_y(dir * el.client_width() as f64 * 0.8, 0.0);
        }
    };
    // Measure once the track is mounted (and its width is known).
    Effect::new(move |_| {
        if track.get().is_some() {
            update();
        }
    });

    view! {
        <div class="home-reviews" class:is-static=move || at_start.get() && at_end.get()>
            <div class="home-reviews-track" node_ref=track on:scroll=move |_| update()
                tabindex="0" role="region" aria-label="家长与学生反馈">
                {REVIEWS.iter().map(|&(who, place, text)| view! {
                    <figure class="home-review">
                        <svg class="home-review-quote" viewBox="0 0 24 24" aria-hidden="true">
                            <path d="M16 3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2 1 1 0 0 1 1 1v1a2 2 0 0 1-2 2 1 1 0 0 0-1 1v2a1 1 0 0 0 1 1 6 6 0 0 0 6-6V5a2 2 0 0 0-2-2z"/>
                            <path d="M5 3a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2 1 1 0 0 1 1 1v1a2 2 0 0 1-2 2 1 1 0 0 0-1 1v2a1 1 0 0 0 1 1 6 6 0 0 0 6-6V5a2 2 0 0 0-2-2z"/>
                        </svg>
                        <blockquote>{text}</blockquote>
                        <figcaption>
                            <span class="home-review-avatar" aria-hidden="true">
                                {who.chars().next().map(String::from)}
                            </span>
                            <span>
                                <strong>{who}</strong>
                                <span>{place}</span>
                            </span>
                        </figcaption>
                    </figure>
                }).collect_view()}
            </div>
            <div class="home-reviews-controls">
                <button type="button" class="home-slider-btn" aria-label="上一条"
                    disabled=move || at_start.get() on:click=move |_| scroll(-1.0)>
                    <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m15 18-6-6 6-6"/></svg>
                </button>
                <button type="button" class="home-slider-btn" aria-label="下一条"
                    disabled=move || at_end.get() on:click=move |_| scroll(1.0)>
                    <svg viewBox="0 0 24 24" aria-hidden="true"><path d="m9 18 6-6-6-6"/></svg>
                </button>
            </div>
        </div>
    }
}

// ----------------------------------------------------------
//  BLOG & NEWS — one shared list component, two flavors.
// ----------------------------------------------------------

#[component]
pub fn BlogPage() -> impl IntoView {
    view! { <PostList kind="blog" heading="Blog"/> }
}

#[component]
pub fn NewsPage() -> impl IntoView {
    view! { <PostList kind="news" heading="News"/> }
}

#[component]
fn PostList(kind: &'static str, heading: &'static str) -> impl IntoView {
    let posts = get_posts(kind);

    view! {
        <Title text=heading/>
        <h2>{heading}</h2>
        {
            if posts.is_empty() {
                view! { <p>"Nothing here yet — check back soon!"</p> }.into_any()
            } else {
                view! {
                    <ul class="post-list">
                        {posts.into_iter().map(|p| view! {
                            <li>
                                <A href=format!("/{}/{}", p.kind, p.slug)>{p.title.to_string()}</A>
                                <span class="date">{p.published_at.to_string()}</span>
                            </li>
                        }).collect_view()}
                    </ul>
                }.into_any()
            }
        }
    }
}

// ----------------------------------------------------------
//  ONE POST — rendered directly from the Leptos view
// ----------------------------------------------------------

#[component]
pub fn PostDetailPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.read().get("slug").unwrap_or_default();
    
    view! {
        {move || match get_post(&slug()) {
            None => view! {
                <p>"This post doesn't exist (anymore)."</p>
                <p><A href="/">"← Back home"</A></p>
            }.into_any(),
            Some(p) => view! {
                <Title text=p.title.to_string()/>
                <article class="post">
                    <h2>{p.title}</h2>
                    <p class="date">{p.published_at}</p>
                    <div class="post-content">
                        {(p.render)()}
                    </div>
                </article>
            }.into_any(),
        }}
    }
}

// ----------------------------------------------------------
//  ADMIN
// ----------------------------------------------------------

#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <Title text="Admin"/>
        <h2>"Admin"</h2>
        <p>"Posts are now managed via code in " <code>"src/posts.rs"</code> "."</p>
        <p>"To create a new post, write a new Leptos component and add it to " <code>"ALL_POSTS"</code> " in " <code>"src/posts.rs"</code> "."</p>
    }
}
