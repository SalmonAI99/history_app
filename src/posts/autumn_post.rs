// ----------------------------------------------------------
//  POST CONTENT COMPONENTS
// ----------------------------------------------------------

use leptos::prelude::*;


pub fn autumn_post() -> AnyView {
    view! {
        <div class="post-content">
            <h2>"Autumn Courses"</h2>
            <p>"Registration is now open!"</p>
            
            // You can easily embed videos, CSS effects, or Leptos interactive components!
            <div style="padding: 20px; background: linear-gradient(45deg, #1e3a8a, #9333ea); border-radius: 8px; margin: 20px 0; text-align: center; color: #fff; box-shadow: 0 4px 15px rgba(0,0,0,0.3);">
                <h3>"✨ Interactive Zone ✨"</h3>
                <p>"This is a raw Leptos component view inside your post."</p>
            </div>
        </div>
    }.into_any()
}
