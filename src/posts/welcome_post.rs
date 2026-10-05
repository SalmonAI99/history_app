//! Leptos view content for the welcome blog post.

use leptos::prelude::*;

pub fn welcome_post() -> AnyView {
    view! {
        <div class="post-content">
            <h2>"Welcome"</h2>
            <p>"Welcome to our new site! Built entirely with Leptos."</p>
            <p>"Below is a completely interactive 3D Voxel Scene representing an Ancient Greek Temple, rendered in real-time."</p>
            
        </div>
    }.into_any()
}
