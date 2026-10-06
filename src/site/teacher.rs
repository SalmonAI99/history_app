//! Teacher biography page and its reusable profile and experience components.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::A;
use super::SiteFooter;

/// Teacher biography landing page, styled independently from the course views.
#[component]
pub fn TeacherPage() -> impl IntoView {
    view! {
        <Title text="贾老师简介 - 读懂历史，看懂世界"/>
        <div class="teacher-page">
            <TeacherProfileHero/>
            <TeacherExperienceSection/>
            <SiteFooter/>
        </div>
    }
}

/// Introduces the teacher with a portrait, teaching focus, and course link.
#[component]
fn TeacherProfileHero() -> impl IntoView {
    view! {
        <section class="teacher-profile-hero" aria-labelledby="teacher-profile-name">
            <div class="teacher-portrait-frame teacher-reveal">
                <img
                    class="teacher-portrait"
                    src="/images/171.jpg"
                    alt="贾老师肖像"
                    width="360"
                    height="480"
                    loading="eager"
                />
            </div>
            <div class="teacher-profile-copy teacher-reveal" style:animation-delay="120ms">
                <span class="teacher-eyebrow">"首席历史讲师"</span>
                <h1 id="teacher-profile-name">"贾老师"</h1>
                <p class="teacher-profile-bio">
                    "深耕历史教育十余年，擅长用充满温度的语言和详实的史料，将枯燥的年代和事件转化为引人入胜的故事。致力于培养孩子们的独立思考能力与广阔的世界观。"
                </p>
                <A href="/classes" attr:class="teacher-course-cta">"浏览课程 →"</A>
            </div>
        </section>
    }
}

/// Lists teaching experience and approach as a responsive, data-driven card grid.
#[component]
fn TeacherExperienceSection() -> impl IntoView {
    view! {
        <section class="teacher-experience-section" aria-labelledby="teacher-experience-title">
            <h2 id="teacher-experience-title" class="teacher-section-title teacher-reveal">
                "深耕与沉淀"
            </h2>
            <div class="teacher-experience-grid">
                {TEACHER_EXPERIENCES.iter().copied().enumerate().map(|(index, experience)| {
                    view! { <TeacherExperienceCard experience=experience index=index/> }
                }).collect_view()}
            </div>
        </section>
    }
}

/// Displays one credential or teaching-method highlight.
#[component]
fn TeacherExperienceCard(experience: TeacherExperience, index: usize) -> impl IntoView {
    let animation_delay = format!("{}ms", 100 + (index % 4) * 100);

    view! {
        <article
            class="teacher-experience-card teacher-reveal"
            style:animation-delay=animation_delay
        >
            <TeacherExperienceIcon icon=experience.icon/>
            <h3>{experience.title}</h3>
            <p>{experience.description}</p>
        </article>
    }
}

/// Renders a small line icon for an experience card.
#[component]
fn TeacherExperienceIcon(icon: TeacherExperienceIconKind) -> impl IntoView {
    let artwork = match icon {
        TeacherExperienceIconKind::Teaching => view! {
            <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
                <circle cx="12" cy="12" r="10"/>
                <polyline points="12 6 12 12 16 14"/>
            </svg>
        }.into_any(),
        TeacherExperienceIconKind::Awards => view! {
            <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
                <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
            </svg>
        }.into_any(),
        TeacherExperienceIconKind::Students => view! {
            <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
                <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
                <circle cx="9" cy="7" r="4"/>
                <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
                <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
            </svg>
        }.into_any(),
        TeacherExperienceIconKind::Courses => view! {
            <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
                <polygon points="12 2 2 7 12 12 22 7 12 2"/>
                <polyline points="2 17 12 22 22 17"/>
                <polyline points="2 12 12 17 22 12"/>
            </svg>
        }.into_any(),
        TeacherExperienceIconKind::Research => view! {
            <svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true">
                <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/>
                <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/>
            </svg>
        }.into_any(),
    };

    view! {
        <div class="teacher-experience-icon">{artwork}</div>
    }
}

/// A small static model keeps the card markup separate from its content.
#[derive(Clone, Copy)]
struct TeacherExperience {
    icon: TeacherExperienceIconKind,
    title: &'static str,
    description: &'static str,
}

#[derive(Clone, Copy)]
enum TeacherExperienceIconKind {
    Teaching,
    Awards,
    Students,
    Courses,
    Research,
}

const TEACHER_EXPERIENCES: &[TeacherExperience] = &[
    TeacherExperience {
        icon: TeacherExperienceIconKind::Teaching,
        title: "十余年一线教学经验",
        description: "从公立学校到线上平台，累计授课超过 5000 小时。深入了解 10–16 岁青少年的认知特点，能够把握不同年龄段的学习难点与兴趣点。",
    },
    TeacherExperience {
        icon: TeacherExperienceIconKind::Awards,
        title: "多次获得教育创新奖项",
        description: "凭借创新的“沉浸式历史教学法”，荣获年度优秀教育创作者及教育领域“启发力”大奖。教学课件被多所学校作为内部培训示范资料。",
    },
    TeacherExperience {
        icon: TeacherExperienceIconKind::Students,
        title: "见证数千名学生成长",
        description: "不仅帮助学生在考试中取得进步，也引导他们建立阅读长文的习惯，培养客观分析复杂事件的思维能力。",
    },
    TeacherExperience {
        icon: TeacherExperienceIconKind::Courses,
        title: "课程内容历经 6 次大迭代",
        description: "坚持打磨教学产品，从文字讲义升级为包含多媒体资料、动态地图与丰富图谱的立体化课程。每次迭代都源于对教学效果的持续改进。",
    },
    TeacherExperience {
        icon: TeacherExperienceIconKind::Research,
        title: "扎实的人文学术背景",
        description: "曾任历史研究所助理研究员，参与古籍整理与历史档案数字化工作。严谨的学术训练帮助课程内容保持准确与扎实。",
    },
];
