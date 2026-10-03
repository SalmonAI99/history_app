#!/usr/bin/env python3
"""
PPTX to Leptos SSR Slide Deck Pipeline (讲义速读 Rust 生成器)

Converts PowerPoint (.pptx) presentations into native Rust source code 
for the Leptos web framework (Server-Side Rendering + WASM Hydration),
instead of generating standalone static JS/HTML.

Features:
- Extracts embedded images and MP4 videos to public/slides/<deck_id>/media/
- Generates strongly typed, zero-allocation Rust static data in src/slides/<deck_id>.rs
- Automatically updates src/slides/mod.rs deck registry and router
- Enables native Leptos client-side SPA navigation via <A href="/slides/<deck_id>">
- Supports full-text search, Table of Contents, and embedded video streaming
"""

import os
import sys
import re
import json
import shutil
import zipfile
import argparse
import xml.etree.ElementTree as ET
from pathlib import Path

try:
    from pptx import Presentation
    from pptx.enum.shapes import MSO_SHAPE_TYPE
except ImportError:
    pass

NS = {
    'p': 'http://schemas.openxmlformats.org/presentationml/2006/main',
    'a': 'http://schemas.openxmlformats.org/drawingml/2006/main',
    'r': 'http://schemas.openxmlformats.org/officeDocument/2006/relationships',
}

PROJECT_ROOT = Path(__file__).resolve().parent.parent
SLIDES_RS_DIR = PROJECT_ROOT / "src" / "slides"
PUBLIC_SLIDES_DIR = PROJECT_ROOT / "public" / "slides"


def slugify(text: str) -> str:
    """Converts a name or title into a safe Rust identifier and URL slug."""
    s = text.strip().lower()
    # Replace non-alphanumeric (allowing latin, digits, pinyin/underscores)
    s = re.sub(r'[\s\-\.]+', '_', s)
    s = re.sub(r'[^\w]', '', s)
    if not s or s[0].isdigit():
        s = "deck_" + s
    return s


def extract_media(pptx_path: Path, media_dir: Path) -> dict:
    """Extracts all embedded media (images, videos) from PPTX zip."""
    media_dir.mkdir(parents=True, exist_ok=True)
    extracted = {}
    with zipfile.ZipFile(pptx_path, "r") as z:
        for item in z.infolist():
            if item.filename.startswith("ppt/media/"):
                basename = os.path.basename(item.filename)
                target = media_dir / basename
                with z.open(item) as src, open(target, "wb") as dst:
                    shutil.copyfileobj(src, dst)
                extracted[basename] = f"media/{basename}"
    return extracted


def parse_presentation(pptx_path: Path, media_dir: Path):
    """Dynamically parses all slides, shapes, text, images, and videos."""
    if 'Presentation' not in globals():
        raise RuntimeError("python-pptx is not installed in the active Python environment.")

    prs = Presentation(str(pptx_path))
    slides_data = []

    for idx, slide in enumerate(prs.slides):
        slide_num = idx + 1
        rels = {rid: rel.target_ref for rid, rel in slide.part.rels.items()}

        slide_title = ""
        paragraphs = []
        media_items = []

        try:
            if slide.shapes.title and slide.shapes.title.text.strip():
                slide_title = slide.shapes.title.text.strip()
        except Exception:
            pass

        for shape in slide.shapes:
            # Video
            root = ET.fromstring(shape._element.xml)
            vf = root.find('.//a:videoFile', NS)

            if vf is not None or shape.shape_type == MSO_SHAPE_TYPE.MEDIA:
                r_link = vf.attrib.get('{http://schemas.openxmlformats.org/officeDocument/2006/relationships}link') if vf is not None else None
                target_video = rels.get(r_link) if r_link else None

                poster = None
                blip = root.find('.//a:blip', NS)
                if blip is not None:
                    r_embed = blip.attrib.get('{http://schemas.openxmlformats.org/officeDocument/2006/relationships}embed')
                    if r_embed and rels.get(r_embed):
                        poster = "media/" + os.path.basename(rels.get(r_embed))

                if target_video:
                    media_items.append({
                        "type": "video",
                        "src": "media/" + os.path.basename(target_video),
                        "poster": poster,
                        "title": shape.name or f"第{slide_num}页视频"
                    })
                continue

            # Picture
            if shape.shape_type == MSO_SHAPE_TYPE.PICTURE:
                blip = root.find('.//a:blip', NS)
                if blip is not None:
                    r_embed = blip.attrib.get('{http://schemas.openxmlformats.org/officeDocument/2006/relationships}embed')
                    if r_embed and rels.get(r_embed):
                        img_filename = os.path.basename(rels.get(r_embed))
                        media_items.append({
                            "type": "image",
                            "src": f"media/{img_filename}",
                            "alt": shape.name or f"第{slide_num}页插图"
                        })
                continue

            # Text
            if shape.has_text_frame:
                is_title_shape = False
                try:
                    if slide.shapes.title and shape.id == slide.shapes.title.id:
                        is_title_shape = True
                except Exception:
                    pass

                for p in shape.text_frame.paragraphs:
                    text = p.text.strip()
                    if not text:
                        continue
                    if is_title_shape and not slide_title:
                        slide_title = text
                        continue
                    paragraphs.append(text)

        if not slide_title:
            if paragraphs:
                first = paragraphs[0]
                slide_title = first if len(first) <= 40 else first[:38] + "…"
            elif media_items:
                media_type = "视频" if any(m["type"] == "video" for m in media_items) else "图像"
                slide_title = f"{media_type}资料"
            else:
                slide_title = f"第 {slide_num} 页"

        slides_data.append({
            "slide_number": slide_num,
            "title": slide_title,
            "paragraphs": paragraphs,
            "media": media_items
        })

    return slides_data


def generate_rust_code(deck_id: str, title: str, slides: list) -> str:
    """Generates Leptos-ready Rust code for the slide deck."""
    out = []
    out.append("// ============================================================")
    out.append(f"//  Generated by pipeline/convert_pptx.py for {deck_id}")
    out.append("//  Leptos SSR Slide Deck — DO NOT EDIT DIRECTLY")
    out.append("// ============================================================")
    out.append("")
    out.append("use leptos::prelude::*;")
    out.append("use crate::slides::model::*;")
    out.append("")
    out.append(f"pub const ID: &str = {json.dumps(deck_id, ensure_ascii=False)};")
    out.append(f"pub const TITLE: &str = {json.dumps(title, ensure_ascii=False)};")
    out.append(f"pub const SLIDE_COUNT: usize = {len(slides)};")
    out.append("")
    out.append("pub const SLIDES: &[Slide] = &[")

    for s in slides:
        num = s["slide_number"]
        t = json.dumps(s["title"], ensure_ascii=False)
        out.append("    Slide {")
        out.append(f"        num: {num},")
        out.append(f"        title: {t},")
        out.append("        paragraphs: &[")
        for p in s.get("paragraphs", []):
            out.append(f"            {json.dumps(p, ensure_ascii=False)},")
        out.append("        ],")
        out.append("        media: &[")
        for m in s.get("media", []):
            m_type = "MediaType::Video" if m.get("type") == "video" else "MediaType::Image"
            src = json.dumps(m.get("src", ""), ensure_ascii=False)
            poster = f'Some({json.dumps(m["poster"], ensure_ascii=False)})' if m.get("poster") else "None"
            cap = m.get("title") or m.get("alt")
            caption = f'Some({json.dumps(cap, ensure_ascii=False)})' if cap else "None"
            out.append("            MediaItem {")
            out.append(f"                media_type: {m_type},")
            out.append(f"                src: {src},")
            out.append(f"                poster: {poster},")
            out.append(f"                caption: {caption},")
            out.append("            },")
        out.append("        ],")
        out.append("    },")

    out.append("];")
    out.append("")
    out.append("pub const DECK: SlideDeck = SlideDeck {")
    out.append("    id: ID,")
    out.append("    title: TITLE,")
    out.append("    slide_count: SLIDE_COUNT,")
    out.append("    slides: SLIDES,")
    out.append("};")
    out.append("")
    out.append("pub const fn get_deck() -> SlideDeck {")
    out.append("    DECK")
    out.append("}")
    out.append("")
    out.append("/// Direct Leptos SSR component for this deck")
    out.append("#[component]")
    out.append("pub fn DeckPage() -> impl IntoView {")
    out.append("    view! {")
    out.append("        <crate::slides::SlideDeckViewer deck=get_deck() />")
    out.append("    }")
    out.append("}")
    out.append("")

    return "\n".join(out)


def update_slides_mod_rs():
    """Scans src/slides/*.rs and updates src/slides/mod.rs to register all decks."""
    deck_modules = []
    for f in sorted(SLIDES_RS_DIR.glob("*.rs")):
        stem = f.stem
        if stem in ("mod", "model", "access"):
            continue
        deck_modules.append(stem)

    mod_rs_path = SLIDES_RS_DIR / "mod.rs"
    
    # We maintain the core logic in mod.rs and inject module imports + ALL_DECKS
    imports = "\n".join([f"pub mod {m};" for m in deck_modules])
    all_decks_items = ",\n    ".join([f"{m}::DECK" for m in deck_modules])

    content = f'''// ============================================================
//  slides/mod.rs — Leptos SSR Slide Decks & Lecture Reader
// ============================================================

pub mod access;
pub mod model;
{imports}

pub use access::*;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{{components::A, hooks::use_params_map}};
pub use model::{{MediaItem, MediaType, Slide, SlideDeck}};

pub const ALL_DECKS: &[SlideDeck] = &[
    {all_decks_items}
];

pub fn find_deck(id: &str) -> Option<SlideDeck> {{
    ALL_DECKS.iter().copied().find(|d| d.id == id)
}}

/// Dynamic Route component for `/slides/:id`
#[component]
pub fn SlideDeckPage() -> impl IntoView {{
    let params = use_params_map();
    let deck_id = move || params.read().get("id").unwrap_or_default();

    view! {{
        {{move || {{
            let id = deck_id();
            match find_deck(&id) {{
                Some(deck) => view! {{
                    <SlideDeckViewer deck=deck />
                }}.into_any(),
                None => view! {{
                    <div class="slides-not-found" style="max-width: 600px; margin: 40px auto; text-align: center; padding: 30px; background: #ffffff; border-radius: 8px; border: 1px solid #e7e2d7;">
                        <h2>"未找到该讲义"</h2>
                        <p style="color: #6b7280; margin: 12px 0 20px 0;">
                            {{format!("不存在 ID 为 \\"{{}}\\" 的讲义速读。", id)}}
                        </p>
                        <A href="/slides" attr:class="button">"← 返回讲义速读列表"</A>
                    </div>
                }}.into_any(),
            }}
        }}}}
    }}
}}

/// Index page for `/slides` listing all available lecture slide decks.
#[component]
pub fn SlidesIndexPage() -> impl IntoView {{
    let unlocked = Resource::new(|| (), |_| async {{ get_unlocked_courses().await.unwrap_or_default() }});

    view! {{
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

            <Transition fallback=move || view! {{ <p class="loading">"正在加载课程列表..."</p> }}>
                {{move || {{
                    let unlocked_list = unlocked.get().unwrap_or_default();
                    view! {{
                        <div class="deck-grid" style="display: grid; gap: 20px;">
                            {{ALL_DECKS.iter().map(|deck| {{
                                let deck_id = deck.id;
                                let deck_title = deck.title;
                                let slide_count = deck.slide_count;
                                let href = format!("/slides/{{}}", deck_id);
                                let is_unlocked = unlocked_list.contains(&deck_id.to_string());

                                view! {{
                                    <div class="deck-card" style="background: #ffffff; border: 1px solid #e7e2d7; border-radius: 10px; padding: 20px; display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap; box-shadow: 0 1px 4px rgba(0,0,0,0.03);">
                                        <div>
                                            <div style="font-weight: 700; font-size: 1.2rem; color: #29251f; margin-bottom: 6px; display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                                                <span>{{deck_title}}</span>
                                                {{if is_unlocked {{
                                                    view! {{
                                                        <span style="background: #dcfce7; color: #15803d; font-size: 0.78rem; font-weight: 600; padding: 2px 8px; border-radius: 4px;">
                                                            "已解锁"
                                                        </span>
                                                    }}.into_any()
                                                }} else {{
                                                    view! {{
                                                        <span style="background: #fef3c7; color: #b45309; font-size: 0.78rem; font-weight: 600; padding: 2px 8px; border-radius: 4px;">
                                                            "🔒 需购买 · ￥29.9"
                                                        </span>
                                                    }}.into_any()
                                                }}}}
                                            </div>
                                            <div style="font-size: 0.88rem; color: #7a6e60; display: flex; gap: 10px; align-items: center; flex-wrap: wrap;">
                                                <span style="background: #eadecd; padding: 2px 8px; border-radius: 4px; font-weight: 600; color: #4b3e2e;">
                                                    {{format!("{{}} 页讲义", slide_count)}}
                                                </span>
                                                <span>{{format!("ID: {{}}", deck_id)}}</span>
                                                <span>"• 含多媒体视频/图片"</span>
                                            </div>
                                        </div>
                                        <div>
                                            <A
                                                href=href
                                                attr:class="button"
                                                attr:style=if is_unlocked {{
                                                    "background: #1d4ed8; color: #ffffff; padding: 8px 18px; border-radius: 6px; text-decoration: none; font-weight: 600; display: inline-block;"
                                                }} else {{
                                                    "background: #d97706; color: #ffffff; padding: 8px 18px; border-radius: 6px; text-decoration: none; font-weight: 600; display: inline-block;"
                                                }}
                                            >
                                                {{if is_unlocked {{ "开始学习 →" }} else {{ "查看讲义 (试读/购买) →" }}}}
                                            </A>
                                        </div>
                                    </div>
                                }}
                            }}).collect_view()}}
                        </div>
                    }}
                }}}}
            </Transition>
        </div>
    }}
}}

/// The core Leptos SSR Slide Viewer Component with Access Control & Locked Preview
#[component]
pub fn SlideDeckViewer(deck: SlideDeck) -> impl IntoView {{
    let deck_id = deck.id.to_string();
    let access = Resource::new(
        move || deck_id.clone(),
        |id| async move {{ check_course_access(id).await.unwrap_or_default() }},
    );
    let buy_action = ServerAction::<BuyCourseAccess>::new();

    // After purchase/unlock action succeeds, refetch access:
    Effect::new(move |_| {{
        if matches!(buy_action.value().get(), Some(Ok(()))) {{
            access.refetch();
        }}
    }});

    let (query, set_query) = signal(String::new());
    let page_title = format!("{{}} — 课程讲义速读", deck.title);

    view! {{
        <Title text=page_title/>
        <div class="slide-deck-viewer" style="max-width: 860px; margin: 0 auto; padding: 24px 16px 60px 16px; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', Georgia, serif; line-height: 1.8; color: #29251f;">
            <header class="deck-header" style="margin-bottom: 24px; padding-bottom: 18px; border-bottom: 2px solid #e7e2d7; background: transparent; color: inherit; text-align: left; display: block;">
                <div class="back-nav" style="margin-bottom: 12px; font-size: 14px;">
                    <A href="/materials" attr:style="color: #6b7280; text-decoration: none;">"← 课程资料库"</A>
                    <span style="color: #cbd5e1; margin: 0 8px;">"|"</span>
                    <A href="/slides" attr:style="color: #6b7280; text-decoration: none;">"全部讲义"</A>
                </div>
                <h1 style="font-size: 28px; font-weight: 700; margin: 0 0 10px 0; color: #1e293b;">
                    {{deck.title}}
                </h1>
                <div class="deck-meta" style="font-size: 14px; color: #6b7280; display: flex; gap: 14px; flex-wrap: wrap; align-items: center;">
                    <span>{{format!("📑 共 {{}} 页讲义", deck.slide_count)}}</span>
                    <span>"🎬 嵌入式视频/图档"</span>
                    <Transition fallback=|| ()>
                        {{move || match access.get() {{
                            Some(st) if st.is_unlocked => {{
                                if st.is_admin {{
                                    view! {{ <span style="background: #e0f2fe; color: #0369a1; padding: 2px 8px; border-radius: 4px; font-weight: 600; font-size: 13px;">"👑 管理员授权"</span> }}.into_any()
                                }} else {{
                                    view! {{ <span style="background: #dcfce7; color: #15803d; padding: 2px 8px; border-radius: 4px; font-weight: 600; font-size: 13px;">"✅ 已解锁完整讲义"</span> }}.into_any()
                                }}
                            }}
                            Some(_) => view! {{
                                <span style="background: #fef3c7; color: #b45309; padding: 2px 8px; border-radius: 4px; font-weight: 600; font-size: 13px;">
                                    "🔒 未解锁 (试读模式)"
                                </span>
                            }}.into_any(),
                            None => ().into_any(),
                        }}}}
                    </Transition>
                </div>
            </header>

            <Transition fallback=move || view! {{ <p class="loading">"正在加载讲义与权限..."</p> }}>
                {{move || match access.get() {{
                    None => ().into_any(),
                    Some(status) => {{
                        let is_unlocked = status.is_unlocked;

                        view! {{
                            <div>
                                // If locked, display the Locked Hero Card with Purchase Callout:
                                {{if !is_unlocked {{
                                    view! {{
                                        <div class="locked-course-card" style="background: #fffdf5; border: 2px solid #f59e0b; border-radius: 12px; padding: 22px 24px; margin-bottom: 28px; box-shadow: 0 4px 12px rgba(245, 158, 11, 0.1);">
                                            <div style="display: flex; gap: 16px; align-items: flex-start; flex-wrap: wrap;">
                                                <div style="font-size: 34px; line-height: 1;">"🔒"</div>
                                                <div style="flex: 1; min-width: 260px;">
                                                    <h3 style="margin: 0 0 6px 0; font-size: 19px; color: #92400e; font-weight: 700;">
                                                        "本课程讲义已锁定 (Course Locked)"
                                                    </h3>
                                                    <p style="margin: 0 0 14px 0; color: #78350f; font-size: 14.5px; line-height: 1.6;">
                                                        {{format!("当前讲义共包含 {{}} 页精讲课件、历史背景深度剖析及随堂原版多媒体视频。未购买学员仅可免费浏览课程目录大纲与第 1 页试读内容。", deck.slide_count)}}
                                                    </p>
                                                    {{if status.is_logged_in {{
                                                        let uname = status.username.clone().unwrap_or_default();
                                                        view! {{
                                                            <div style="background: #ffffff; border: 1px solid #fde68a; border-radius: 8px; padding: 14px 18px; display: flex; justify-content: space-between; align-items: center; gap: 16px; flex-wrap: wrap;">
                                                                <div>
                                                                    <div style="font-weight: 600; color: #29251f; font-size: 14px;">
                                                                        {{format!("当前学员账户：{{}}", uname)}}
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
                                                        }}.into_any()
                                                    }} else {{
                                                        view! {{
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
                                                        }}.into_any()
                                                    }}}}
                                                </div>
                                            </div>
                                        </div>
                                    }}.into_any()
                                }} else {{
                                    ().into_any()
                                }}}}

                                // Search bar
                                <div class="search-wrap" style="margin: 20px 0; position: sticky; top: 12px; z-index: 50; padding: 6px 0; background: #f5f1e8;">
                                    <input
                                        type="text"
                                        class="search-box"
                                        placeholder=if is_unlocked {{
                                            "🔍 快速搜索本讲内容（输入关键词，实时过滤）..."
                                        }} else {{
                                            "🔍 搜索课程大纲（购买解锁后可全文检索全部讲义与正文）..."
                                        }}
                                        style="width: 100%; padding: 12px 16px; font-size: 15px; border: 1px solid #cbd5e1; border-radius: 8px; background: #ffffff; box-shadow: 0 2px 6px rgba(0,0,0,0.04); outline: none; font-family: inherit;"
                                        on:input=move |ev| set_query.set(event_target_value(&ev))
                                        prop:value=query
                                    />
                                </div>

                                // Table of Contents
                                <details class="toc-card" open style="background: #ffffff; border: 1px solid #e7e2d7; border-radius: 8px; padding: 14px 18px; margin-bottom: 30px;">
                                    <summary style="font-weight: 600; cursor: pointer; color: #1d4ed8; user-select: none; display: flex; justify-content: space-between; align-items: center;">
                                        <span>"📑 目录大纲 (点击展开/收起)"</span>
                                        {{if !is_unlocked {{
                                            view! {{
                                                <span style="font-size: 12px; color: #b45309; font-weight: normal;">
                                                    "第 1 页试读 · 第 2-90 页已锁定 🔒"
                                                </span>
                                            }}.into_any()
                                        }} else {{
                                            ().into_any()
                                        }}}}
                                    </summary>
                                    <ul class="toc-list" style="list-style: none; margin: 12px 0 0 0; padding: 0 0 0 4px; max-height: 280px; overflow-y: auto;">
                                        {{deck.slides.iter().map(|s| {{
                                            let num = s.num;
                                            let t = s.title;
                                            let has_video = s.media.iter().any(|m| m.media_type == MediaType::Video);
                                            let anchor = format!("#slide-{{}}", num);

                                            view! {{
                                                <li style="padding: 4px 0; font-size: 14px;">
                                                    <a href=anchor style="color: #29251f; text-decoration: none; display: flex; align-items: center; gap: 6px;">
                                                        <span style="color: #6b7280; font-size: 12px; min-width: 24px;">{{format!("{{}}.", num)}}</span>
                                                        <span style="flex: 1;">{{t}}</span>
                                                        {{if !is_unlocked {{
                                                            if num == 1 {{
                                                                view! {{
                                                                    <span style="background: #e0f2fe; color: #0284c7; padding: 1px 6px; border-radius: 3px; font-size: 11px; font-weight: 600;">
                                                                        "试读"
                                                                    </span>
                                                                }}.into_any()
                                                            }} else {{
                                                                view! {{
                                                                    <span style="color: #d97706; font-size: 12px;" title="需购买解锁">
                                                                        "🔒"
                                                                    </span>
                                                                }}.into_any()
                                                            }}
                                                        }} else {{
                                                            ().into_any()
                                                        }}}}
                                                        {{if has_video {{
                                                            view! {{ <span style="margin-left: 4px;">"🎬"</span> }}.into_any()
                                                        }} else {{
                                                            ().into_any()
                                                        }}}}
                                                    </a>
                                                </li>
                                            }}
                                        }}).collect_view()}}
                                    </ul>
                                </details>

                                // Slide Items
                                <main id="lecture-list">
                                    {{move || {{
                                        let q = query.get().trim().to_lowercase();
                                        let slides = deck.slides;
                                        let filtered: Vec<&Slide> = if q.is_empty() {{
                                            slides.iter().collect()
                                        }} else {{
                                            slides.iter().filter(|s| {{
                                                s.title.to_lowercase().contains(&q)
                                                    || (is_unlocked && s.paragraphs.iter().any(|p| p.to_lowercase().contains(&q)))
                                                    || format!("{{}}", s.num).contains(&q)
                                            }}).collect()
                                        }};

                                        if filtered.is_empty() {{
                                            view! {{
                                                <div style="text-align: center; padding: 40px; color: #6b7280; background: #ffffff; border-radius: 8px; border: 1px dashed #cbd5e1;">
                                                    "🔍 没有找到与关键词匹配的讲义大纲。"
                                                </div>
                                            }}.into_any()
                                        }} else {{
                                            view! {{
                                                <div>
                                                    {{filtered.into_iter().map(|s| {{
                                                        let is_slide_locked = !is_unlocked && s.num > 1;
                                                        render_slide(deck.id, s, is_slide_locked)
                                                    }}).collect_view()}}
                                                </div>
                                            }}.into_any()
                                        }}
                                    }}}}
                                </main>
                            </div>
                        }}.into_any()
                    }}
                }}}}
            </Transition>
        </div>
    }}
}}

fn render_slide(deck_id: &'static str, s: &Slide, is_locked: bool) -> impl IntoView {{
    let num = s.num;
    let title = s.title;
    let slide_id = format!("slide-{{}}", num);

    view! {{
        <article
            class="lecture-item"
            id=slide_id
            style="background: #ffffff; border: 1px solid #e7e2d7; border-radius: 10px; padding: 24px; margin-bottom: 24px; box-shadow: 0 1px 4px rgba(0,0,0,0.03); scroll-margin-top: 80px;"
        >
            <div class="item-header" style="display: flex; align-items: baseline; gap: 12px; margin-bottom: 16px; padding-bottom: 10px; border-bottom: 1px dashed #e7e2d7;">
                {{if is_locked {{
                    view! {{
                        <span class="slide-badge" style="font-size: 12px; font-weight: 700; color: #92400e; background: #fef3c7; padding: 2px 8px; border-radius: 4px; flex-shrink: 0;">
                            {{format!("第 {{}} 页 🔒", num)}}
                        </span>
                    }}.into_any()
                }} else if num == 1 {{
                    view! {{
                        <span class="slide-badge" style="font-size: 12px; font-weight: 700; color: #0369a1; background: #e0f2fe; padding: 2px 8px; border-radius: 4px; flex-shrink: 0;">
                            {{format!("第 {{}} 页 (试读)", num)}}
                        </span>
                    }}.into_any()
                }} else {{
                    view! {{
                        <span class="slide-badge" style="font-size: 12px; font-weight: 700; color: #4b5563; background: #f1f5f9; padding: 2px 8px; border-radius: 4px; flex-shrink: 0;">
                            {{format!("第 {{}} 页", num)}}
                        </span>
                    }}.into_any()
                }}}}

                <h2 style="font-size: 20px; font-weight: 700; flex: 1; color: #1e3a8a; line-height: 1.4; margin: 0;">
                    {{title}}
                </h2>
                <a href="#toc" class="top-link" style="font-size: 12px; color: #6b7280; text-decoration: none;">"↑ 目录"</a>
            </div>

            {{if is_locked {{
                // Locked Slide Placeholder:
                view! {{
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
                }}.into_any()
            }} else {{
                view! {{
                    <div>
                        // Media
                        {{s.media.iter().map(|m| {{
                            let src_url = format!("/slides-media/{{}}/{{}}", deck_id, m.src);
                            match m.media_type {{
                                MediaType::Video => {{
                                    let poster_url = m.poster.map(|p| format!("/slides-media/{{}}/{{}}", deck_id, p));
                                    view! {{
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
                                                {{format!("▶ 视频资料（第 {{}} 页）", num)}}
                                            </div>
                                        </div>
                                    }}.into_any()
                                }}
                                MediaType::Image => {{
                                    let caption = m.caption.unwrap_or("");
                                    view! {{
                                        <div class="lecture-media image-box" style="margin: 16px 0; border-radius: 6px; overflow: hidden; background: #000000; display: flex; flex-direction: column; align-items: center;">
                                            <img
                                                src=src_url
                                                alt=caption
                                                loading="lazy"
                                                style="max-width: 100%; height: auto; max-height: 480px; display: block; margin: 0 auto;"
                                            />
                                        </div>
                                    }}.into_any()
                                }}
                            }}
                        }}).collect_view()}}

                        // Paragraphs
                        <div class="item-content" style="font-size: 16px; line-height: 1.8;">
                            {{if s.paragraphs.is_empty() {{
                                view! {{
                                    <p class="empty-note" style="color: #6b7280; font-style: italic; font-size: 14px; margin: 0;">
                                        "（本页为图档/视频内容）"
                                    </p>
                                }}.into_any()
                            }} else {{
                                view! {{
                                    <div>
                                        {{s.paragraphs.iter().map(|p| {{
                                            view! {{
                                                <p style="margin-bottom: 12px; text-align: justify;">
                                                    {{*p}}
                                                </p>
                                            }}
                                        }}).collect_view()}}
                                    </div>
                                }}.into_any()
                            }}}}
                        </div>
                    </div>
                }}.into_any()
            }}}}
        </article>
    }}
}}
'''
    mod_rs_path.write_text(content, encoding="utf-8")
    print(f"Updated {mod_rs_path} with decks: {deck_modules}")


def process_pptx(pptx_file: Path, deck_id: str = None) -> tuple[str, str, int]:
    """Processes a single PPTX into Leptos SSR Rust file and public media directory."""
    deck_name = pptx_file.stem
    if not deck_id:
        # Default mapping for specific known courses or slugify
        if "巴以冲突1" in deck_name:
            deck_id = "israel_palestine_1"
        elif "巴以冲突" in deck_name and "2" in deck_name:
            deck_id = "israel_palestine_2"
        else:
            deck_id = slugify(deck_name)

    print(f"\nProcessing PPTX: {pptx_file.name}")
    print(f"  Deck ID: {deck_id}")

    public_dir = PUBLIC_SLIDES_DIR / deck_id
    media_dir = public_dir / "media"
    public_dir.mkdir(parents=True, exist_ok=True)
    media_dir.mkdir(parents=True, exist_ok=True)

    print("  [1/3] Extracting media...")
    extract_media(pptx_file, media_dir)

    print("  [2/3] Parsing slides...")
    slides = parse_presentation(pptx_file, media_dir)
    print(f"  Parsed {len(slides)} slides.")

    # Save lecture.json
    json_path = public_dir / "lecture.json"
    with open(json_path, "w", encoding="utf-8") as f:
        json.dump({"title": deck_name, "slides": slides}, f, ensure_ascii=False, indent=2)

    print("  [3/3] Generating Rust SSR component...")
    rust_code = generate_rust_code(deck_id, deck_name, slides)
    SLIDES_RS_DIR.mkdir(parents=True, exist_ok=True)
    rs_path = SLIDES_RS_DIR / f"{deck_id}.rs"
    rs_path.write_text(rust_code, encoding="utf-8")
    print(f"  Generated {rs_path}")

    return deck_id, deck_name, len(slides)


def process_from_json(json_path: Path, deck_id: str = None):
    """Generates Leptos SSR Rust code from an existing lecture.json file."""
    with open(json_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    title = data.get("title", "Lecture")
    slides = data.get("slides", [])
    if not deck_id:
        deck_id = json_path.parent.name

    print(f"\nGenerating Rust from JSON: {json_path}")
    print(f"  Deck ID: {deck_id}, Title: {title}, Slides: {len(slides)}")

    rust_code = generate_rust_code(deck_id, title, slides)
    SLIDES_RS_DIR.mkdir(parents=True, exist_ok=True)
    rs_path = SLIDES_RS_DIR / f"{deck_id}.rs"
    rs_path.write_text(rust_code, encoding="utf-8")
    print(f"  Generated {rs_path}")


def main():
    parser = argparse.ArgumentParser(description="PPTX to Leptos SSR Slide Deck Pipeline.")
    parser.add_argument("pptx", nargs="?", help="Path to input .pptx file")
    parser.add_argument("--deck-id", help="Custom identifier for the deck (used in URL /slides/<deck_id>)")
    parser.add_argument("--from-json", help="Path to an existing lecture.json file")
    parser.add_argument("--all", action="store_true", help="Process all PPTX files in storage/ or regenerate from public/slides/")
    args = parser.parse_args()

    if args.from_json:
        process_from_json(Path(args.from_json), args.deck_id)
        update_slides_mod_rs()
        return

    if args.all or not args.pptx:
        # Search storage for PPTX files
        storage_dir = PROJECT_ROOT / "storage"
        pptx_files = list(storage_dir.glob("*.pptx"))

        if pptx_files:
            print(f"Found {len(pptx_files)} presentation(s) in storage/:")
            for p in pptx_files:
                process_pptx(p)
        else:
            # Fallback to existing public/slides/*/lecture.json
            json_files = list(PUBLIC_SLIDES_DIR.glob("*/lecture.json"))
            print(f"No .pptx found; regenerating from {len(json_files)} existing lecture.json files:")
            for j in json_files:
                process_from_json(j)

        update_slides_mod_rs()
        print("\nAll slide decks successfully generated in Rust for Leptos SSR!")
        return

    pptx_path = Path(args.pptx)
    if not pptx_path.exists():
        print(f"Error: PPTX file not found: {pptx_path}")
        sys.exit(1)

    process_pptx(pptx_path, args.deck_id)
    update_slides_mod_rs()
    print("\nSlide deck successfully generated in Rust for Leptos SSR!")


if __name__ == "__main__":
    main()
