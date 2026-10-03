# PPTX to Leptos SSR Slide Deck Pipeline (讲义速读 Rust 转换流)

This pipeline transforms PowerPoint (`.pptx`) presentations stored in `storage/` directly into **native Rust files in the Leptos framework (SSR + WASM Hydration)** instead of standalone HTML/JS files.

This allows lecture decks to integrate seamlessly into BookReader's single-page router (`leptos_router`), sharing the site header, theme, session context, and instant client-side routing.

---

## 🚀 Quick Start / How to Run

### 1. Process all PPTX presentations in `storage/`:
```bash
.venv/bin/python pipeline/convert_pptx.py --all
```

### 2. Process a specific PPTX file:
```bash
.venv/bin/python pipeline/convert_pptx.py storage/巴以冲突1.pptx --deck-id israel_palestine_1
```

### 3. Regenerate Rust code from existing `lecture.json`:
```bash
.venv/bin/python pipeline/convert_pptx.py --from-json public/slides/israel_palestine_1/lecture.json
```

Once processed, the new deck is instantly registered and accessible via Leptos router:
- **Lecture Index Page:** `http://127.0.0.1:3000/slides`
- **Specific Deck Route:** `http://127.0.0.1:3000/slides/israel_palestine_1`
- **Internal SPA Navigation:** Navigable from the site navbar **"讲义速读"** (`/slides`) or via the **Materials** page (`/materials`).

---

## 🛠️ Pipeline Architecture

```
storage/<deck>.pptx (or public/slides/<deck>/lecture.json)
       │
       ▼
[pipeline/convert_pptx.py]
       │
       ├── 1. Media Extraction
       │      • Extracts embedded images and MP4 videos to public/slides/<deck_id>/media/
       │      • Axum serves media at /slides-media/<deck_id>/... with HTTP 206 range streaming
       │
       ├── 2. Shape, Geometry & Text Parsing
       │      • Extracts titles, typography, paragraphs, alignments, and video/poster relationships
       │
       ├── 3. Rust Leptos SSR Code Generation
       │      • Generates src/slides/<deck_id>.rs with compile-time zero-allocation static slices
       │      • Updates src/slides/mod.rs to register new decks in ALL_DECKS and routing
       │
       └── 4. Native Leptos SPA Integration
              • Routed at /slides and /slides/:id
              • Real-time reactive client-side search (via Leptos signals)
              • Anchor-based Table of Contents
              • Video player with streaming and posters
```

---

## 🌟 Benefits of Leptos SSR Generation

1. **Zero Hydration Mismatch**: Server and client compile the exact same static Rust data structures and views.
2. **Native Router Integration**: Works with `<A href="/slides/...">` and browser history, with no page reloads.
3. **Reactive Search**: Full-text instant filtering powered by Leptos signals (`query` / `set_query`).
4. **Range Video Streaming**: Axum serves `/slides-media/...` with partial content support for video seeking.
