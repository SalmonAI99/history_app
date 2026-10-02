// ============================================================
//  world.rs — Book World: a tiny 2D grid you walk around in.
//
//  How it works (deliberately the simplest thing that feels alive):
//   - the world is a 16x10 grid. You are a colored square; arrows and
//     WASD move you one tile. Your key moves a LOCAL signal instantly
//     (no waiting for the server), then tells the server.
//   - the server keeps ONE row per player (position + last message)
//     in world_users. Rows not refreshed for 5 minutes are "gone".
//   - the multiplayer bit: every open page POLLS the server every
//     1.5 seconds ("who is where, saying what?") and draws everyone.
//   - the top row (y = 0) holds one bookshelf pad per book: step on
//     it and a "jump into this book" link appears, and others see
//     you standing at that shelf.
//   If this outgrows polling (many players, live typing), swap the
//   poll loop for a websocket. The data model here doesn't change.
// ============================================================

use crate::auth::User;
use leptos::prelude::*;
use leptos_router::components::A;
use serde::{Deserialize, Serialize};

// The grid. The server clamps to these bounds and the browser draws
// with them — one definition, no drift.
pub const COLS: i64 = 16;
pub const ROWS: i64 = 10;

/// A bookshelf pad's x tile (pads sit on the top row, y = 0).
/// Deterministic from the book's position in the list so the server
/// and the browser always agree without storing anything.
fn pad_x(index: i64) -> i64 {
    1 + index * 4
}

fn clamp(value: i64, lo: i64, hi: i64) -> i64 {
    value.max(lo).min(hi)
}

fn clamp_x(x: i64) -> i64 {
    clamp(x, 0, COLS - 1)
}
fn clamp_y(y: i64) -> i64 {
    clamp(y, 0, ROWS - 1)
}

// `web_sys` is only available in the browser build. Keeping this small check
// behind the hydrate feature lets the server build compile as well.
#[cfg(feature = "hydrate")]
fn is_text_entry_target(target: leptos::web_sys::EventTarget) -> bool {
    use leptos::wasm_bindgen::JsCast;
    target
        .dyn_ref::<leptos::web_sys::HtmlElement>()
        .map(|el| matches!(el.tag_name().as_str(), "INPUT" | "TEXTAREA"))
        .unwrap_or(false)
}

// ----------------------------------------------------------
//  SHARED TYPES (server -> browser)
// ----------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookStub {
    pub id: i64,
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    pub username: String,
    pub x: i64,
    pub y: i64,
    pub book_id: i64, // shelf you're standing at (0 = none)
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub you_x: i64,
    pub you_y: i64,
    pub you_name: String,
    pub people: Vec<Person>, // everyone currently here, including you
    pub books: Vec<BookStub>,
}

/// Chat text hygiene: trim, cap at 120 CHARACTERS (chars() counts
/// characters, not bytes — "啊" is one char, so Chinese is safe).
fn clean_message(s: &str) -> String {
    s.trim().chars().take(120).collect()
}

// ----------------------------------------------------------
//  SERVER FUNCTIONS
// ----------------------------------------------------------

/// Everything the world page needs, one call. Polling this every
/// 1.5s doubles as the "I'm still here" signal (presence refresh).
#[server]
pub async fn get_world() -> Result<WorldSnapshot, ServerFnError> {
    let user = crate::workspace::ssr::require_user().await?;
    let mut conn = crate::db::db().await?;

    // Heartbeat: creates your row on first visit, refreshes it after.
    sqlx::query(
        "INSERT INTO world_users (user_id, username) VALUES (?, ?)
         ON CONFLICT (user_id) DO UPDATE SET updated_at = datetime('now')",
    )
    .bind(user.id)
    .bind(&user.username)
    .execute(&mut conn)
    .await?;

    let (you_x, you_y): (i64, i64) =
        sqlx::query_as("SELECT x, y FROM world_users WHERE user_id = ?")
            .bind(user.id)
            .fetch_one(&mut conn)
            .await?;

    let rows: Vec<(String, i64, i64, i64, String)> = sqlx::query_as(
        "SELECT username, x, y, book_id, message FROM world_users
         WHERE updated_at > datetime('now', '-5 minutes')
         ORDER BY username",
    )
    .fetch_all(&mut conn)
    .await?;

    let people = rows
        .into_iter()
        .map(|(username, x, y, book_id, message)| Person {
            username,
            x: clamp_x(x),
            y: clamp_y(y),
            book_id,
            message,
        })
        .collect();

    let book_rows: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, title FROM books ORDER BY id")
            .fetch_all(&mut conn)
            .await?;
    let books = book_rows
        .into_iter()
        .map(|(id, title)| BookStub { id, title })
        .collect();

    Ok(WorldSnapshot {
        you_x: clamp_x(you_x),
        you_y: clamp_y(you_y),
        you_name: user.username,
        people,
        books,
    })
}

/// Move one tile. The client clamps already; the server clamps AGAIN
/// (never trust the client — the stage 9 rule, in a game).
#[server]
pub async fn move_player(x: i64, y: i64) -> Result<(), ServerFnError> {
    let user = crate::workspace::ssr::require_user().await?;
    let mut conn = crate::db::db().await?;

    let x = clamp_x(x);
    let y = clamp_y(y);

    // Standing on a shelf pad (top row) marks which book you're at.
    let book_id = if y == 0 {
        let pads: Vec<(i64,)> =
            sqlx::query_as("SELECT id FROM books ORDER BY id")
                .fetch_all(&mut conn)
                .await?;
        pads.iter()
            .enumerate()
            .find(|(i, _)| pad_x(*i as i64) == x)
            .map(|(_, (id,))| *id)
            .unwrap_or(0)
    } else {
        0
    };

    sqlx::query(
        "INSERT INTO world_users (user_id, username, x, y, book_id)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT (user_id) DO UPDATE SET
             x = excluded.x,
             y = excluded.y,
             book_id = excluded.book_id,
             updated_at = datetime('now')",
    )
    .bind(user.id)
    .bind(&user.username)
    .bind(x)
    .bind(y)
    .bind(book_id)
    .execute(&mut conn)
    .await?;
    Ok(())
}

/// Say something — replaces your latest message bubble.
#[server]
pub async fn say(message: String) -> Result<(), ServerFnError> {
    let user = crate::workspace::ssr::require_user().await?;

    let message = clean_message(&message);
    if message.is_empty() {
        return Err(ServerFnError::ServerError("Say something first.".into()));
    }

    let mut conn = crate::db::db().await?;
    sqlx::query(
        "INSERT INTO world_users (user_id, username, message) VALUES (?, ?, ?)
         ON CONFLICT (user_id) DO UPDATE SET
             message = excluded.message,
             updated_at = datetime('now')",
    )
    .bind(user.id)
    .bind(&user.username)
    .bind(&message)
    .execute(&mut conn)
    .await?;
    Ok(())
}

// ----------------------------------------------------------
//  VIEW PIECES
// ----------------------------------------------------------

/// A stable color per player: same name always gets the same hue.
fn hue(name: &str) -> u32 {
    name.bytes().map(|b| b as u32).sum::<u32>() % 360
}

fn tile_style(x: i64, y: i64, hue: u32) -> String {
    // One tile is 100/16 % wide and 100/10 % tall — the avatar
    // simply occupies one tile's rectangle.
    format!(
        "left:{}%; top:{}%; background:hsl({hue}, 70%, 58%)",
        x as f64 * (100.0 / COLS as f64),
        y as f64 * (100.0 / ROWS as f64)
    )
}

/// One avatar: a colored square + name tag + optional latest message.
fn avatar_view(
    name: String,
    x: i64,
    y: i64,
    message: String,
    you: bool,
) -> impl IntoView {
    let class = if you { "avatar you" } else { "avatar" };
    let style = tile_style(x, y, hue(&name));
    let has_msg = !message.is_empty();

    view! {
        <div class=class style=style>
            {if has_msg {
                view! { <span class="bubble">{message}</span> }.into_any()
            } else {
                ().into_any()
            }}
            <span class="tag">
                {name.clone()}
                {you.then(|| view! { <span class="me">" (you)"</span> })}
            </span>
        </div>
    }
}

// ----------------------------------------------------------
//  THE PAGE
// ----------------------------------------------------------

#[component]
pub fn WorldPage() -> impl IntoView {
    let user = expect_context::<Resource<Option<User>>>();
    let world = Resource::new(|| (), |_| get_world());
    let move_player = ServerAction::<MovePlayer>::new();
    let say = ServerAction::<Say>::new();
    let say_ref = NodeRef::<leptos::html::Input>::new();

    // Your position as the page sees it right now (optimistic moves
    // change this INSTANTLY; the server confirms within a moment).
    let you_pos = RwSignal::new((3i64, 3i64));
    // Adopt the server position only ONCE (first snapshot); after that
    // your optimistic moves would otherwise rubber-band on every poll.
    let synced_once = RwSignal::new(false);

    // 1. Poll every 1.5s — the whole multiplayer mechanism.
    Effect::new(move |_| {
        if let Ok(handle) = leptos::leptos_dom::helpers::set_interval_with_handle(
            move || world.refetch(),
            std::time::Duration::from_millis(1500),
        ) {
            on_cleanup(move || handle.clear());
        }
    });

    // 2. First snapshot: adopt the position the server has for you.
    Effect::new(move |_| {
        if let Some(Ok(snap)) = world.get() {
            if !synced_once.get() {
                synced_once.set(true);
                you_pos.set((snap.you_x, snap.you_y));
            }
        }
    });

    // 3. After speaking: clear the box and refresh.
    Effect::new(move |_| {
        if matches!(say.value().get(), Some(Ok(()))) {
            if let Some(i) = say_ref.get() {
                i.set_value("");
            }
            world.refetch();
        }
    });

    // 4. Global keyboard handler: move locally NOW, tell the server after.
    //    Global (window) listener = no "click the field first", and the
    //    1.5s poll re-render can't knock keys dead by stealing focus.
    Effect::new(move |_| {
        let handle = window_event_listener(leptos::ev::keydown, move |ev| {
            // don't walk while typing a chat message
            #[cfg(feature = "hydrate")]
            if ev.target().is_some_and(is_text_entry_target) {
                return;
            }
            let (dx, dy) = match ev.key().as_str() {
                "ArrowUp" | "w" | "W" => (0, -1),
                "ArrowDown" | "s" | "S" => (0, 1),
                "ArrowLeft" | "a" | "A" => (-1, 0),
                "ArrowRight" | "d" | "D" => (1, 0),
                _ => return,
            };
            ev.prevent_default();
            let (x, y) = you_pos.get_untracked();
            let (nx, ny) = (clamp_x(x + dx), clamp_y(y + dy));
            if (nx, ny) != (x, y) {
                you_pos.set((nx, ny));
                move_player.dispatch(MovePlayer { x: nx, y: ny });
            }
        });
        on_cleanup(move || handle.remove());
    });

    view! {
        <h2>"Book World"</h2>
        <p class="note-hint">
            "Walk with arrows or WASD — anywhere on the page. Stand on a bookshelf 📚 to jump into that book."
        </p>
        <Transition fallback=move || view! { <p class="loading">"Loading..."</p> }>
            {move || match user.get() {
                None => ().into_any(),
                Some(None) => view! {
                    <p class="note-hint">
                        "Please " <A href="/login">"log in"</A> " to enter the world."
                    </p>
                }.into_any(),
                Some(Some(u)) => view! {
                    {move || match world.get() {
                        None => view! { <p class="loading">"Loading..."</p> }.into_any(),
                        Some(Err(_)) => view! {
                            <p>"Could not load the world."</p>
                        }.into_any(),
                        Some(Ok(snap)) => game_view(snap, u.username.clone(), you_pos).into_any(),
                    }}
                    <ChatZone say=say say_ref=say_ref/>
                }.into_any(),
            }}
        </Transition>
    }
}

// ----------------------------------------------------------
//  THE GAME FIELD + CHAT BAR
// ----------------------------------------------------------

fn game_view(
    snap: WorldSnapshot,
    my_name: String,
    you_pos: RwSignal<(i64, i64)>,
) -> impl IntoView {
    let books = snap.books.clone();

    view! {
        {move || {
            let (yx, yy) = you_pos.get();
            let on_pad = books
                .iter()
                .enumerate()
                .find(|(i, _)| pad_x(*i as i64) == yx && yy == 0)
                .map(|(_, b)| b.clone());

            view! {
                <div class="game" tabindex="0">
                    // bookshelf pads on the top row
                    {books.iter().enumerate().map(|(i, b)| {
                        let left = pad_x(i as i64) as f64 * (100.0 / COLS as f64);
                        view! {
                            <div class="pad" style=format!("left:{left}%; top:0%") title=b.title.clone()>
                                "📚"
                                <span class="pad-tag">{b.title.clone()}</span>
                            </div>
                        }
                    }).collect_view()}
                    // everyone currently here
                    {snap.people.iter().map(|p| {
                        let is_you = p.username == my_name;
                        if is_you {
                            // YOUR avatar follows the optimistic signal
                            let (ax, ay) = you_pos.get();
                            avatar_view(p.username.clone(), ax, ay, p.message.clone(), true).into_any()
                        } else {
                            avatar_view(p.username.clone(), p.x, p.y, p.message.clone(), false).into_any()
                        }
                    }).collect_view()}
                </div>
                // stepping on a shelf reveals the jump link
                {match on_pad {
                    Some(book) => view! {
                        <div class="jump-overlay">
                            "Standing at " <strong>{book.title.clone()}</strong> ". "
                            <A attr:class="button" href=format!("/book/{}/chapter/1", book.id)>
                                "📖 Jump to reading →"
                            </A>
                        </div>
                    }.into_any(),
                    None => ().into_any(),
                }}
            }
        }}
    }
}

/// The chat bar under the field.
#[component]
fn ChatZone(say: ServerAction<Say>, say_ref: NodeRef<leptos::html::Input>) -> impl IntoView {
    view! {
        <div class="chat-zone">
            <ActionForm action=say attr:class="chat-bar">
                <input type="text" name="message" node_ref=say_ref
                    placeholder="Say something nice..." maxlength="120" required/>
                <button type="submit">"Say it"</button>
            </ActionForm>
            <ActionForm action=say attr:class="chat-bar inline">
                <input type="hidden" name="message" value="👋 Hello!"/>
                <button type="submit">"👋 Greet"</button>
            </ActionForm>
        </div>
    }
}

// ----------------------------------------------------------
//  UNIT TESTS
// ----------------------------------------------------------

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn trims_and_truncates_by_characters() {
        assert_eq!(clean_message("  hello  "), "hello");
        assert_eq!(clean_message(&"啊".repeat(200)).chars().count(), 120);
        assert_eq!(clean_message("   "), "");
    }

    #[test]
    fn clamps_positions_to_the_grid() {
        assert_eq!(clamp_x(-5), 0);
        assert_eq!(clamp_x(999), COLS - 1);
        assert_eq!(clamp_x(7), 7);
        assert_eq!(clamp_y(-1), 0);
        assert_eq!(clamp_y(999), ROWS - 1);
    }

    #[test]
    fn shelf_pads_spread_safely() {
        assert_eq!(pad_x(0), 1);
        assert_eq!(pad_x(1), 5);
        // Even the third shelf stays inside the grid (1 + 2*4 = 9).
        assert_eq!(pad_x(2), 9);
        assert!(pad_x(2) < COLS);
    }
}
