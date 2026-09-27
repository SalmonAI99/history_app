// ============================================================
//  main.rs — the SERVER entry point.
//  This is the program you run: it listens on http://127.0.0.1:3000
//  and serves your app. It only compiles with the "ssr" feature.
// ============================================================

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tower_http::trace::TraceLayer;
    use tracing::info;
    use bookreader::app::*;

    // ----- TELEMETRY: structured logging -----
    // One line per event, with timestamps and key=value fields —
    // exactly what you grep at 2am when something broke.
    // Control verbosity with RUST_LOG (default: info + HTTP requests):
    //   RUST_LOG=debug cargo leptos watch
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .init();

    // Create the database tables and fill them on first run.
    bookreader::db::init()
        .await
        .expect("could not initialize the database");
    info!("database ready");

    // Read the [package.metadata.leptos] settings from Cargo.toml.
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Ask the App component which URLs it can render.
    let routes = generate_route_list(App);

    // Build the Axum web server:
    let app = Router::new()
        // ...hand it the Leptos routes (pages + server functions)...
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        // ...serve static files (CSS, favicon, the WASM bundle) as a fallback...
        .fallback(leptos_axum::file_and_error_handler(shell))
        // ...and log EVERY request: method, path, status, latency.
        .layer(TraceLayer::new_for_http())
        .with_state(leptos_options);

    info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

// When compiling for the browser (WASM), there is no server main().
#[cfg(not(feature = "ssr"))]
pub fn main() {}
