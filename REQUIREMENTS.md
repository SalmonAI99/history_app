# Book Reader / Course Platform — Requirements

## Product overview

A bilingual (Chinese/English) website for an **education business** selling a
**reading course series for children**, operating **primarily in mainland China**.
It is both a **marketing site** (to attract parents) and a **learning platform**
(for paying students), with personal storage and AI-assisted reading features.

The customers are **parents** (accounts, payments); the readers are **children
in 5th grade of primary school (~10–11 years old)**. This shapes the product:
- Simple, forgiving UI: large text, big buttons, minimal navigation
- AI explanations written for a 10-year-old reading level (simple words,
  encouraging tone — set in the prompt, one place, from our server)
- Content appropriate for that age (chapter books)
- Minimal data collected about children (privacy laws are strict for minors)

## Functional requirements

### Reading app (core)
- Read books online, chapter by chapter
- User accounts (signup / login)
- Per-user reading **progress** saved and restored
- Per-user **notes and writings** saved

### Personal storage space ("their own storage, similarly accessed")
- Each user has a private workspace: **writings, notes, images**
- Strictly private: every query scoped by `user_id`; files served only
  after an access check (never a public static folder)

### Marketing / public site
- Home page pitching the course series
- **Teacher info** page
- **Blog posts** and **news** (admin/teacher publishes, Markdown content)
- Public zone must be **SEO-friendly** (server-rendered HTML)

### Members-only area (paid users)
- Access to **videos** and **special materials** (PDFs, worksheets, etc.)
- Access rule: logged in **and** paid subscription active
- Roles: `student` vs `admin` (teacher publishes content/materials)

### AI features (native and fast)
- **Explain this passage** — student selects text → plain-language explanation (live, streamed)
- **Ask-the-book tutor chat** — AI answers using the book's content as context (live, streamed)
- **Chapter summaries** — pre-generated once, cached in DB (instant, zero per-user cost;
  teacher can review/edit before publishing since content is for children)
- Provider: a China-available, OpenAI-compatible API (e.g. **DeepSeek** / Qwen)

## Platform requirements

- **Cross-platform, convenient access** — one codebase for phones + desktops
- **Phone "app"** via **PWA** (Add to Home Screen; no separate native apps;
  wrap later with Capacitor/Tauri only if app-store presence is needed)
- **Bilingual: Chinese (primary) + English**
  - UI strings translated in code (language toggle, cookie persistence)
  - Content (blog/news) stored per-language in the DB
- Mobile-friendly, responsive CSS throughout

## China-specific constraints

- **Payments:** WeChat Pay / Alipay (QR-code "Native" flow), **not Stripe**.
  Requires a Chinese business license (营业执照) for merchant accounts.
- **No YouTube/Vimeo** (blocked): self-host videos (MP4 + HTTP range streaming
  behind access checks); later migrate to Alibaba OSS + CDN when bandwidth demands it.
- **AI providers:** OpenAI/Anthropic not usable → DeepSeek/Qwen (OpenAI-compatible).
- **No Google services** (fonts, reCAPTCHA, analytics all blocked) — system fonts,
  simple rate-limiting for signup protection.
- **Hosting/ICP:** mainland hosting requires **ICP备案** (needs Chinese business
  entity); alternative is hosting in **Hong Kong** (no filing, good latency).
- WeChat in-app browser can't install PWAs — instruct users to open in system browser.

## Technical decisions (with rationale)

| Decision | Choice | Why |
|---|---|---|
| Stack | **Leptos full-stack** (Rust → WASM frontend + Axum backend, one project) | Single language (user knows Rust), shared types, server functions, SSR for SEO |
| Database | **SQLite** via SQLx | Zero ops, fast for read-heavy workload, free; migrate to Postgres only if: multiple servers, measured write contention, or vector search needed |
| Passwords/sessions | bcrypt hashes + session-token cookies | Simple, no card data ever touched |
| Video | Self-hosted MP4 + range requests first; OSS+CDN later | Vimeo/YouTube blocked; avoids HLS complexity at v1 |
| Phone app | PWA | Zero rewrite; native wrap deferred |
| AI perf | Streaming + pre-generation/caching | Feels instant; cached AI = free AI |

## Roadmap (14 stages)

| # | Stage | Status |
|---|---|---|
| 1 | Hello Leptos — components, view!, signals | ✅ done |
| 2 | Chapter pager in Rust — signals, events, lists, mobile CSS | ✅ done |
| 3 | Server functions — load book from server (incl. real-browser test harness) | ✅ done |
| 4 | Router — multiple pages and URLs (library + reader, URL-as-state) | ✅ done |
| 5 | SQLite — schema: users+roles, documents, files, posts(+lang), subscriptions | ✅ done |
| 6 | Auth — signup/login, sessions, cookies (bcrypt, session tokens, 8/8 tests) | ✅ done |
| 7 | ~~Bilingual~~ — **cut for simplicity** (2026-09); posts table keeps `lang` column for a possible comeback | ➖ skipped |
| 8 | Public site — home, teacher, blog, news + admin publishing | ✅ done |
| 9 | Workspace — progress + writings/notes (user-scoped) | ✅ done |
| 10 | File storage — uploads, private serving + access-controlled video | ⬜ |
| 11 | Payments (WeChat Pay/Alipay QR) + members-only materials | ⬜ |
| 12 | PWA — manifest + service worker + phone install | ⬜ |
| 13 | Deployment — HK or mainland+ICP, domain, HTTPS | ⬜ |
| 14 | AI features — summaries (cached), explain-passage + tutor chat (streaming) | ⬜ |

*Last updated: after Stage 3.*
