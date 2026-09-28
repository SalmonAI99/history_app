// ============================================================
//  lib.rs — the shared library used by BOTH builds.
//  Right now it just exposes the `app` module and contains
//  the browser entry point (`hydrate`).
// ============================================================

pub mod app;
pub mod auth;
pub mod book;
pub mod posts;
pub mod site;
pub mod workspace;

// The database layer only exists in the server build —
// the browser must never touch the database directly.
#[cfg(feature = "ssr")]
pub mod db;

// This function is the BROWSER entry point. When the WASM module
// loads in the user's browser, this runs first. It "hydrates" the
// page: it walks the HTML the server already sent and attaches
// interactivity (signals, event handlers) to it.
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
