use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashSet;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, WebSocket, MessageEvent, AudioContext};
use leptos::prelude::*;
use super::models::{ClientMessage, ServerMessage, PlayerState, PlayerClass};

thread_local! {
    pub static WS_INSTANCE: RefCell<Option<WebSocket>> = RefCell::new(None);
    pub static AUDIO_CTX: RefCell<Option<AudioContext>> = RefCell::new(None);
    pub static SOUND_MUTED: RefCell<bool> = RefCell::new(false);
}

pub fn send_client_msg(msg: ClientMessage) {
    WS_INSTANCE.with(|ws_ref| {
        if let Some(ws) = ws_ref.borrow().as_ref() {
            if ws.ready_state() == WebSocket::OPEN {
                if let Ok(txt) = serde_json::to_string(&msg) {
                    let _ = ws.send_with_str(&txt);
                }
            }
        }
    });
}

pub fn play_sound(sound_type: &str) {
    SOUND_MUTED.with(|m| {
        if *m.borrow() { return; }
        AUDIO_CTX.with(|actx_ref| {
            if actx_ref.borrow().is_none() {
                if let Ok(ctx) = AudioContext::new() {
                    *actx_ref.borrow_mut() = Some(ctx);
                }
            }
            if let Some(ctx) = actx_ref.borrow().as_ref() {
                let _ = ctx.resume();
                let now = ctx.current_time();
                if let (Ok(osc), Ok(gain)) = (ctx.create_oscillator(), ctx.create_gain()) {
                    let _ = osc.connect_with_audio_node(&gain);
                    let _ = gain.connect_with_audio_node(&ctx.destination());

                    match sound_type {
                        "step" => {
                            osc.set_type(web_sys::OscillatorType::Sine);
                            let _ = osc.frequency().set_value_at_time(120.0, now);
                            let _ = osc.frequency().exponential_ramp_to_value_at_time(70.0, now + 0.04);
                            let _ = gain.gain().set_value_at_time(0.03, now);
                            let _ = gain.gain().linear_ramp_to_value_at_time(0.001, now + 0.04);
                            let _ = osc.start();
                            let _ = osc.stop_with_when(now + 0.04);
                        }
                        "dialogue" => {
                            osc.set_type(web_sys::OscillatorType::Triangle);
                            let _ = osc.frequency().set_value_at_time(440.0, now);
                            let _ = osc.frequency().exponential_ramp_to_value_at_time(659.25, now + 0.12);
                            let _ = gain.gain().set_value_at_time(0.08, now);
                            let _ = gain.gain().linear_ramp_to_value_at_time(0.005, now + 0.15);
                            let _ = osc.start();
                            let _ = osc.stop_with_when(now + 0.15);
                        }
                        "victory" => {
                            osc.set_type(web_sys::OscillatorType::Triangle);
                            let _ = osc.frequency().set_value_at_time(523.25, now);
                            let _ = osc.frequency().set_value_at_time(659.25, now + 0.08);
                            let _ = osc.frequency().set_value_at_time(783.99, now + 0.16);
                            let _ = osc.frequency().set_value_at_time(1046.50, now + 0.24);
                            let _ = gain.gain().set_value_at_time(0.12, now);
                            let _ = gain.gain().linear_ramp_to_value_at_time(0.005, now + 0.45);
                            let _ = osc.start();
                            let _ = osc.stop_with_when(now + 0.45);
                        }
                        "portal" => {
                            osc.set_type(web_sys::OscillatorType::Sine);
                            let _ = osc.frequency().set_value_at_time(220.0, now);
                            let _ = osc.frequency().exponential_ramp_to_value_at_time(880.0, now + 0.22);
                            let _ = gain.gain().set_value_at_time(0.08, now);
                            let _ = gain.gain().linear_ramp_to_value_at_time(0.001, now + 0.25);
                            let _ = osc.start();
                            let _ = osc.stop_with_when(now + 0.25);
                        }
                        _ => {}
                    }
                }
            }
        });
    });
}

// Particle for ambient atmosphere
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    size: f64,
    alpha: f64,
}

// Landmark NPC
struct NpcData {
    name: &'static str,
    title: &'static str,
    x: f64,
    y: f64,
    icon: &'static str,
    color: &'static str,
}

// Teleport Portal
struct PortalData {
    name: &'static str,
    x: f64,
    y: f64,
    target_scene: &'static str,
    target_x: f64,
    target_y: f64,
    color: &'static str,
}

pub fn init_game(
    players_sig: RwSignal<Vec<PlayerState>>,
    current_question_sig: RwSignal<Option<String>>,
    current_scene_sig: RwSignal<String>,
    canvas: HtmlCanvasElement,
) {
    #[cfg(feature = "hydrate")]
    console_error_panic_hook::set_once();

    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    
    let ctx = match canvas.get_context("2d") {
        Ok(Some(c)) => match c.dyn_into::<CanvasRenderingContext2d>() {
            Ok(ctx) => ctx,
            Err(_) => return,
        },
        _ => return,
    };

    let my_id = Rc::new(RefCell::new("local-init".to_string()));

    // Immediately insert local initial player so canvas is NEVER blank while connecting
    if players_sig.get_untracked().is_empty() {
        players_sig.update(|p| {
            p.push(PlayerState {
                id: "local-init".to_string(),
                x: 100.0,
                y: 330.0,
                color: "#2563eb".to_string(),
                class: PlayerClass::Explorer,
                level: 1,
                knowledge: 0,
                scene: "overview".to_string(),
            });
        });
    }

    // Connect WebSocket safely without crashing if network fails
    let protocol = if window.location().protocol().unwrap_or_default() == "https:" { "wss:" } else { "ws:" };
    let host = window.location().host().unwrap_or_else(|_| "localhost:3000".to_string());
    let ws_url = format!("{}//{}/ws/simulator", protocol, host);
    web_sys::console::log_1(&JsValue::from_str(&format!("Connecting simulator WS: {}", ws_url)));

    if let Ok(ws) = WebSocket::new(&ws_url) {
        WS_INSTANCE.with(|ws_ref| {
            *ws_ref.borrow_mut() = Some(ws.clone());
        });

        let my_id_clone = my_id.clone();
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                if let Ok(msg) = serde_json::from_str::<ServerMessage>(&txt.as_string().unwrap()) {
                    players_sig.update(|players| {
                        match &msg {
                            ServerMessage::Welcome { id } => {
                                // Update local ID and replace local-init placeholder
                                let old_id = my_id_clone.borrow().clone();
                                *my_id_clone.borrow_mut() = id.clone();
                                if let Some(local_p) = players.iter_mut().find(|p| p.id == old_id) {
                                    local_p.id = id.clone();
                                }
                            }
                            ServerMessage::StateSync { players: new_players } => {
                                *players = new_players.clone();
                            }
                            ServerMessage::PlayerJoined { player } => {
                                if let Some(existing) = players.iter_mut().find(|p| p.id == player.id) {
                                    *existing = player.clone();
                                } else {
                                    players.push(player.clone());
                                }
                            }
                            ServerMessage::PlayerLeft { id } => {
                                players.retain(|p| p.id != *id);
                            }
                            ServerMessage::PlayerMoved { id, x, y } => {
                                if let Some(p) = players.iter_mut().find(|p| p.id == *id) {
                                    p.x = *x;
                                    p.y = *y;
                                }
                            }
                            ServerMessage::PlayerUpdated { player } => {
                                if let Some(p) = players.iter_mut().find(|p| p.id == player.id) {
                                    *p = player.clone();
                                }
                            }
                            ServerMessage::ShowSlideQuestion { .. } => {}
                        }
                    });
                    
                    if let ServerMessage::ShowSlideQuestion { target_id, question, .. } = msg {
                        if target_id == *my_id_clone.borrow() {
                            play_sound("dialogue");
                            current_question_sig.set(Some(question));
                        }
                    }
                }
            }
        });
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
    }

    // Initialize 40 ambient particles
    let mut particles_vec = Vec::new();
    for i in 0..40 {
        particles_vec.push(Particle {
            x: (i as f64 * 21.0) % 840.0,
            y: (i as f64 * 37.0) % 460.0,
            vx: 0.6 + (i % 5) as f64 * 0.25,
            vy: ((i % 7) as f64 - 3.0) * 0.2,
            size: 1.5 + (i % 3) as f64,
            alpha: 0.35 + (i % 4) as f64 * 0.15,
        });
    }
    let particles = Rc::new(RefCell::new(particles_vec));

    // Smooth Keyboard State Tracking
    let keys_down = Rc::new(RefCell::new(HashSet::<String>::new()));
    {
        let keys_down_clone = keys_down.clone();
        let keydown_cb = Closure::<dyn FnMut(_)>::new(move |e: web_sys::KeyboardEvent| {
            let key = e.key();
            if matches!(key.as_str(), "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "w" | "s" | "a" | "d" | "W" | "S" | "A" | "D" | " " | "e" | "E") {
                e.prevent_default();
            }
            if matches!(key.as_str(), "e" | "E" | " ") {
                send_client_msg(ClientMessage::Interact);
            } else {
                keys_down_clone.borrow_mut().insert(key);
            }
        });
        window.add_event_listener_with_callback("keydown", keydown_cb.as_ref().unchecked_ref()).unwrap();
        keydown_cb.forget();

        let keys_down_up = keys_down.clone();
        let keyup_cb = Closure::<dyn FnMut(_)>::new(move |e: web_sys::KeyboardEvent| {
            let key = e.key();
            keys_down_up.borrow_mut().remove(&key);
        });
        window.add_event_listener_with_callback("keyup", keyup_cb.as_ref().unchecked_ref()).unwrap();
        keyup_cb.forget();
    }

    // Render loop state
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let move_throttle = Rc::new(RefCell::new(0u32));
    let step_sound_counter = Rc::new(RefCell::new(0u32));
    
    *g.borrow_mut() = Some(Closure::new(move || {
        let w = canvas.width() as f64;
        let h = canvas.height() as f64;
        let active_scene = current_scene_sig.get_untracked();
        let current_players = players_sig.get_untracked();
        let local_id = my_id.borrow().clone();

        // Find my current player state
        let my_player = current_players.iter().find(|p| p.id == local_id);
        let my_class = my_player.map(|p| p.class.clone()).unwrap_or(PlayerClass::Explorer);

        // Movement Speed Bonus: Explorer moves faster (9.0 vs 6.8)
        let speed = if my_class == PlayerClass::Explorer { 9.0 } else { 6.8 };

        // Process movement input throttled to ~20Hz
        let mut tick = move_throttle.borrow_mut();
        *tick = (*tick + 1) % 3;
        if *tick == 0 {
            let mut dx = 0.0;
            let mut dy = 0.0;
            let kd = keys_down.borrow();
            if kd.contains("ArrowUp") || kd.contains("w") || kd.contains("W") { dy -= speed; }
            if kd.contains("ArrowDown") || kd.contains("s") || kd.contains("S") { dy += speed; }
            if kd.contains("ArrowLeft") || kd.contains("a") || kd.contains("A") { dx -= speed; }
            if kd.contains("ArrowRight") || kd.contains("d") || kd.contains("D") { dx += speed; }
            if dx != 0.0 || dy != 0.0 {
                send_client_msg(ClientMessage::Move { dx, dy });
                // Play soft footstep click periodically
                let mut sc = step_sound_counter.borrow_mut();
                *sc = (*sc + 1) % 5;
                if *sc == 0 {
                    play_sound("step");
                }
            }
        }

        // ==========================================
        // SCENE DATA CONFIGURATIONS
        // ==========================================
        let mut npcs: Vec<NpcData> = Vec::new();
        let mut portals: Vec<PortalData> = Vec::new();

        match active_scene.as_str() {
            "overview" => {
                npcs = vec![
                    NpcData { name: "摩西", title: "旷野向导", x: 120.0, y: 310.0, icon: "🧙‍♂️", color: "#d97706" },
                    NpcData { name: "绿洲长老", title: "西奈向导", x: 270.0, y: 200.0, icon: "🧔", color: "#ca8a04" },
                    NpcData { name: "所罗门王", title: "第一圣殿", x: 440.0, y: 110.0, icon: "👑", color: "#eab308" },
                    NpcData { name: "中世纪学者", title: "欧洲城镇", x: 690.0, y: 110.0, icon: "🧑‍🏫", color: "#2563eb" },
                    NpcData { name: "守望老人", title: "西墙哭墙", x: 430.0, y: 330.0, icon: "👴", color: "#64748b" },
                    NpcData { name: "卓别林", title: "和平讲坛", x: 700.0, y: 320.0, icon: "🎩", color: "#0f172a" },
                ];
                portals = vec![
                    PortalData { name: "进入西奈圣山营地", x: 140.0, y: 230.0, target_scene: "sinai", target_x: 100.0, target_y: 220.0, color: "#d97706" },
                    PortalData { name: "进入所罗门第一圣殿", x: 440.0, y: 55.0, target_scene: "temple", target_x: 420.0, target_y: 370.0, color: "#eab308" },
                    PortalData { name: "进入中世纪欧洲城镇", x: 690.0, y: 55.0, target_scene: "europe", target_x: 420.0, target_y: 370.0, color: "#3b82f6" },
                    PortalData { name: "进入和平探索讲坛", x: 700.0, y: 240.0, target_scene: "peace", target_x: 420.0, target_y: 370.0, color: "#10b981" },
                ];
            }
            "sinai" => {
                npcs = vec![
                    NpcData { name: "摩西 · 圣山授法", title: "十诫法典", x: 200.0, y: 220.0, icon: "🧙‍♂️", color: "#d97706" },
                    NpcData { name: "营地先师", title: "羊皮书讲堂", x: 620.0, y: 180.0, icon: "📜", color: "#b45309" },
                    NpcData { name: "寻水长老", title: "旷野甘泉", x: 420.0, y: 330.0, icon: "💧", color: "#0284c7" },
                ];
                portals = vec![
                    PortalData { name: "← 返回大地图", x: 90.0, y: 80.0, target_scene: "overview", target_x: 120.0, target_y: 310.0, color: "#f59e0b" },
                ];
            }
            "temple" => {
                npcs = vec![
                    NpcData { name: "至圣所祭司", title: "约柜守护者", x: 420.0, y: 130.0, icon: "✨", color: "#eab308" },
                    NpcData { name: "贤王所罗门", title: "智慧法庭", x: 200.0, y: 300.0, icon: "👑", color: "#ca8a04" },
                    NpcData { name: "推罗工匠大师", title: "雪松木构", x: 640.0, y: 300.0, icon: "🌲", color: "#15803d" },
                ];
                portals = vec![
                    PortalData { name: "← 返回大地图", x: 90.0, y: 80.0, target_scene: "overview", target_x: 440.0, target_y: 130.0, color: "#f59e0b" },
                ];
            }
            "europe" => {
                npcs = vec![
                    NpcData { name: "排字先驱", title: "古典知识翻印", x: 220.0, y: 200.0, icon: "📖", color: "#2563eb" },
                    NpcData { name: "公社长老", title: "隔都互助社", x: 420.0, y: 120.0, icon: "🤝", color: "#475569" },
                    NpcData { name: "跨国汇兑商人", title: "信用汇票", x: 620.0, y: 280.0, icon: "🧮", color: "#0d9488" },
                ];
                portals = vec![
                    PortalData { name: "← 返回大地图", x: 90.0, y: 80.0, target_scene: "overview", target_x: 690.0, target_y: 130.0, color: "#f59e0b" },
                ];
            }
            "peace" => {
                npcs = vec![
                    NpcData { name: "查理·卓别林", title: "大独裁者演说", x: 220.0, y: 230.0, icon: "🎩", color: "#0f172a" },
                    NpcData { name: "和平守望者", title: "橄榄林誓言", x: 620.0, y: 230.0, icon: "🕊️", color: "#059669" },
                ];
                portals = vec![
                    PortalData { name: "← 返回大地图", x: 90.0, y: 80.0, target_scene: "overview", target_x: 700.0, target_y: 300.0, color: "#f59e0b" },
                ];
            }
            _ => {}
        }

        // ==========================================
        // 1. RENDER BACKGROUND & ARCHITECTURE BY SCENE
        // ==========================================
        match active_scene.as_str() {
            "sinai" => {
                // Sinai Desert: Dramatic dunes & starry night sky
                ctx.set_fill_style_str("#b45309");
                ctx.fill_rect(0.0, 0.0, w, h);
                ctx.set_fill_style_str("#d97706");
                ctx.fill_rect(0.0, 60.0, w, h - 60.0);
                ctx.set_fill_style_str("#f59e0b");
                ctx.fill_rect(0.0, 150.0, w, h - 150.0);

                // Mount Sinai jagged peaks in background
                ctx.set_fill_style_str("#78350f");
                ctx.begin_path();
                ctx.move_to(120.0, 160.0);
                ctx.line_to(200.0, 70.0);
                ctx.line_to(280.0, 160.0);
                ctx.fill();
                ctx.begin_path();
                ctx.move_to(250.0, 160.0);
                ctx.line_to(330.0, 90.0);
                ctx.line_to(410.0, 160.0);
                ctx.fill();

                // Encampment Tents
                draw_tent(&ctx, 620.0, 180.0, "#fef3c7");
                draw_tent(&ctx, 520.0, 220.0, "#fed7aa");
                draw_tent(&ctx, 680.0, 260.0, "#fde68a");

                // Campfire with glowing embers
                ctx.set_fill_style_str("rgba(239, 68, 68, 0.7)");
                ctx.begin_path();
                let _ = ctx.arc(580.0, 230.0, 14.0, 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();
                ctx.set_fill_style_str("#fbbf24");
                ctx.begin_path();
                let _ = ctx.arc(580.0, 230.0, 8.0, 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();
            }
            "temple" => {
                // Solomon's Temple Inner Sanctuary (Cedarwood & Marble Tile)
                ctx.set_fill_style_str("#451a03"); // Dark Cedar ceiling/walls
                ctx.fill_rect(0.0, 0.0, w, h);
                // Inlaid marble floor
                ctx.set_fill_style_str("#fef3c7");
                ctx.fill_rect(40.0, 50.0, w - 80.0, h - 90.0);
                
                // Floor Tile Grid
                ctx.set_stroke_style_str("#fde68a");
                ctx.set_line_width(1.0);
                for tx in (40..800).step_by(50) {
                    ctx.begin_path();
                    ctx.move_to(tx as f64, 50.0);
                    ctx.line_to(tx as f64, h - 40.0);
                    ctx.stroke();
                }

                // 4 Monumental Gold-Capped Cedar Pillars
                draw_pillar(&ctx, 160.0, 90.0, h - 140.0);
                draw_pillar(&ctx, 280.0, 90.0, h - 140.0);
                draw_pillar(&ctx, 560.0, 90.0, h - 140.0);
                draw_pillar(&ctx, 680.0, 90.0, h - 140.0);

                // Golden Ark Sanctum Dais
                ctx.set_fill_style_str("#d97706");
                ctx.fill_rect(380.0, 90.0, 80.0, 50.0);
                ctx.set_fill_style_str("#fbbf24");
                ctx.fill_rect(395.0, 105.0, 50.0, 25.0);
            }
            "europe" => {
                // Medieval Town (Cobblestone Street & Timber Houses)
                ctx.set_fill_style_str("#334155");
                ctx.fill_rect(0.0, 0.0, w, h);
                ctx.set_fill_style_str("#64748b");
                ctx.fill_rect(50.0, 50.0, w - 100.0, h - 100.0);

                // Cobblestone Road
                ctx.set_fill_style_str("#94a3b8");
                ctx.fill_rect(100.0, 160.0, w - 200.0, 140.0);

                // Timber Houses on Top & Bottom
                draw_medieval_house(&ctx, 180.0, 60.0, "#78350f");
                draw_medieval_house(&ctx, 380.0, 60.0, "#713f12");
                draw_medieval_house(&ctx, 580.0, 60.0, "#451a03");
                draw_medieval_house(&ctx, 280.0, 320.0, "#854d0e");
                draw_medieval_house(&ctx, 500.0, 320.0, "#78350f");
            }
            "peace" => {
                // Modern Peace Forum (Marble Plaza & Blue Sky)
                ctx.set_fill_style_str("#0284c7");
                ctx.fill_rect(0.0, 0.0, w, 120.0);
                ctx.set_fill_style_str("#f8fafc");
                ctx.fill_rect(0.0, 120.0, w, h - 120.0);

                // Central Peace Fountain
                ctx.set_fill_style_str("#38bdf8");
                ctx.begin_path();
                let _ = ctx.arc(420.0, 250.0, 48.0, 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();
                ctx.set_stroke_style_str("#0284c7");
                ctx.set_line_width(3.0);
                ctx.stroke();

                // Olive Grove Trees
                draw_olive_tree(&ctx, 580.0, 200.0);
                draw_olive_tree(&ctx, 660.0, 250.0);
                draw_olive_tree(&ctx, 600.0, 300.0);
            }
            _ => {
                // "overview" Default Historical World
                ctx.set_fill_style_str("#fde68a");
                ctx.fill_rect(0.0, 0.0, 320.0, h);

                // Nile River
                ctx.set_fill_style_str("#38bdf8");
                ctx.begin_path();
                ctx.move_to(30.0, 0.0);
                ctx.bezier_curve_to(80.0, 150.0, 20.0, 300.0, 70.0, h);
                ctx.line_to(0.0, h);
                ctx.line_to(0.0, 0.0);
                ctx.fill();

                // Middle: Canaan / Jerusalem
                ctx.set_fill_style_str("#e2e8f0");
                ctx.fill_rect(320.0, 0.0, 260.0, h);
                ctx.set_fill_style_str("#dcfce7");
                ctx.fill_rect(340.0, 40.0, 220.0, h - 80.0);

                // East: Europe
                ctx.set_fill_style_str("#cbd5e1");
                ctx.fill_rect(580.0, 0.0, w - 580.0, h);

                // Boundary Dashes
                ctx.set_stroke_style_str("#a8a29e");
                ctx.set_line_width(2.0);
                ctx.begin_path();
                ctx.move_to(320.0, 0.0); ctx.line_to(320.0, h);
                ctx.move_to(580.0, 0.0); ctx.line_to(580.0, h);
                ctx.stroke();

                // Region Labels
                ctx.set_font("bold 12px sans-serif");
                ctx.set_text_align("left");
                ctx.set_fill_style_str("#b45309");
                let _ = ctx.fill_text("🇪🇬 尼罗河与西奈旷野", 85.0, 24.0);
                ctx.set_fill_style_str("#15803d");
                let _ = ctx.fill_text("🏛️ 迦南 · 耶路撒冷圣山", 360.0, 24.0);
                ctx.set_fill_style_str("#1e3a8a");
                let _ = ctx.fill_text("🏰 中世纪欧洲城镇与和平讲坛", 610.0, 24.0);

                // Ancient Trading Road
                ctx.set_stroke_style_str("#e6ccaa");
                ctx.set_line_width(16.0);
                ctx.set_line_cap("round");
                ctx.begin_path();
                ctx.move_to(110.0, 310.0);
                ctx.quadratic_curve_to(240.0, 270.0, 280.0, 210.0);
                ctx.quadratic_curve_to(340.0, 150.0, 440.0, 120.0);
                ctx.quadratic_curve_to(560.0, 110.0, 680.0, 110.0);
                ctx.quadratic_curve_to(600.0, 250.0, 440.0, 330.0);
                ctx.quadratic_curve_to(550.0, 340.0, 690.0, 320.0);
                ctx.stroke();

                // Landmarks
                draw_pyramid(&ctx, 70.0, 250.0);
                draw_temple_facade(&ctx, 440.0, 75.0);
                draw_western_wall(&ctx, 430.0, 290.0);
            }
        }

        // ==========================================
        // 2. AMBIENT ATMOSPHERIC PARTICLE SYSTEM
        // ==========================================
        {
            let mut parts = particles.borrow_mut();
            for p in parts.iter_mut() {
                p.x += p.vx;
                p.y += p.vy;
                if p.x > w { p.x = 0.0; }
                if p.x < 0.0 { p.x = w; }
                if p.y > h { p.y = 0.0; }
                if p.y < 0.0 { p.y = h; }

                let color = match active_scene.as_str() {
                    "sinai" => format!("rgba(245, 158, 11, {})", p.alpha), // Desert Sand
                    "temple" => format!("rgba(253, 224, 71, {})", p.alpha), // Golden Sparks
                    "europe" => format!("rgba(148, 163, 184, {})", p.alpha), // Mist
                    "peace" => format!("rgba(255, 255, 255, {})", p.alpha), // Light motes
                    _ => format!("rgba(217, 119, 6, {})", p.alpha * 0.7),
                };
                ctx.set_fill_style_str(&color);
                ctx.begin_path();
                let _ = ctx.arc(p.x, p.y, p.size, 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();
            }
        }

        // ==========================================
        // 3. TELEPORT PORTALS (World Navigation)
        // ==========================================
        for pt in portals.iter() {
            // Swirling portal halo
            ctx.set_fill_style_str("rgba(255, 255, 255, 0.25)");
            ctx.begin_path();
            let _ = ctx.arc(pt.x, pt.y, 22.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();

            ctx.set_stroke_style_str(pt.color);
            ctx.set_line_width(2.5);
            ctx.stroke();

            ctx.set_font("14px sans-serif");
            ctx.set_text_align("center");
            let _ = ctx.fill_text("🌀", pt.x, pt.y + 5.0);

            ctx.set_font("bold 10px sans-serif");
            ctx.set_fill_style_str("#1c1917");
            let _ = ctx.fill_text(pt.name, pt.x, pt.y + 24.0);
        }

        // ==========================================
        // 4. HISTORICAL GUIDE NPCS
        // ==========================================
        for npc in npcs.iter() {
            ctx.set_fill_style_str("rgba(250, 204, 21, 0.35)");
            ctx.begin_path();
            let _ = ctx.arc(npc.x, npc.y, 20.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();

            // Robe Body
            ctx.set_fill_style_str(npc.color);
            ctx.fill_rect(npc.x - 8.0, npc.y - 12.0, 16.0, 24.0);

            // Head
            ctx.set_fill_style_str("#fed7aa");
            ctx.begin_path();
            let _ = ctx.arc(npc.x, npc.y - 18.0, 7.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();

            // Head icon
            ctx.set_font("14px sans-serif");
            ctx.set_text_align("center");
            let _ = ctx.fill_text(npc.icon, npc.x, npc.y - 28.0);

            // Name label
            ctx.set_font("bold 10px sans-serif");
            ctx.set_fill_style_str("#1c1917");
            let _ = ctx.fill_text(npc.name, npc.x, npc.y + 22.0);
            ctx.set_font("9px sans-serif");
            ctx.set_fill_style_str("#78716c");
            let _ = ctx.fill_text(npc.title, npc.x, npc.y + 32.0);
        }

        // ==========================================
        // 5. PLAYERS IN SAME SCENE
        // ==========================================
        let mut nearby_prompt: Option<String> = None;
        let mut portal_trigger: Option<(&'static str, f64, f64)> = None;

        for p in current_players.iter() {
            // Only render players who are currently in the SAME scene
            if p.scene != active_scene { continue; }

            // Shadow
            ctx.set_fill_style_str("rgba(0, 0, 0, 0.25)");
            ctx.begin_path();
            let _ = ctx.arc(p.x, p.y + 11.0, 9.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();

            // Robe Body
            ctx.set_fill_style_str(&p.color);
            ctx.fill_rect(p.x - 7.0, p.y - 10.0, 14.0, 20.0);

            // Head
            ctx.set_fill_style_str("#fed7aa");
            ctx.begin_path();
            let _ = ctx.arc(p.x, p.y - 16.0, 7.0, 0.0, std::f64::consts::PI * 2.0);
            ctx.fill();

            // Turban / Class Highlight
            ctx.set_fill_style_str(&p.color);
            ctx.begin_path();
            let _ = ctx.arc(p.x, p.y - 19.0, 6.0, std::f64::consts::PI, std::f64::consts::PI * 2.0);
            ctx.fill();

            // Local Player checks
            if p.id == local_id {
                // Golden Halo
                ctx.set_stroke_style_str("#facc15");
                ctx.set_line_width(2.0);
                ctx.begin_path();
                let _ = ctx.arc(p.x, p.y + 2.0, 16.0, 0.0, std::f64::consts::PI * 2.0);
                ctx.stroke();

                // Check distance to NPCs
                for npc in npcs.iter() {
                    let dist = ((p.x - npc.x).powi(2) + (p.y - npc.y).powi(2)).sqrt();
                    if dist < 65.0 {
                        nearby_prompt = Some(format!("按 [E] 或 [空格] 与 {} 对话", npc.name));
                        break;
                    }
                }

                // Check distance to Portals
                for pt in portals.iter() {
                    let dist = ((p.x - pt.x).powi(2) + (p.y - pt.y).powi(2)).sqrt();
                    if dist < 32.0 {
                        portal_trigger = Some((pt.target_scene, pt.target_x, pt.target_y));
                        nearby_prompt = Some(format!("按 [E] 穿行前往: {}", pt.name));
                        break;
                    }
                }
            }
        }

        // Handle Portal Step Collision Trigger
        if let Some((target_scene, tx, ty)) = portal_trigger {
            let kd = keys_down.borrow();
            if kd.contains("e") || kd.contains("E") || kd.contains(" ") {
                play_sound("portal");
                current_scene_sig.set(target_scene.to_string());
                send_client_msg(ClientMessage::ChangeScene { scene: target_scene.to_string(), x: tx, y: ty });
            }
        }

        // ==========================================
        // 6. ATMOSPHERIC TORCHLIGHT VIGNETTE
        // ==========================================
        if let Some(my_p) = my_player {
            let vision_radius = if my_class == PlayerClass::Explorer { 320.0 } else { 220.0 };
            if let Ok(grad) = ctx.create_radial_gradient(my_p.x, my_p.y, 40.0, my_p.x, my_p.y, vision_radius) {
                let _ = grad.add_color_stop(0.0, "rgba(0, 0, 0, 0.0)");
                let _ = grad.add_color_stop(1.0, "rgba(12, 10, 9, 0.38)");
                ctx.set_fill_style_canvas_gradient(&grad);
                ctx.fill_rect(0.0, 0.0, w, h);
            }
        }

        // ==========================================
        // 7. INTERACTION HUD PROMPT
        // ==========================================
        if let Some(text) = nearby_prompt {
            ctx.set_fill_style_str("rgba(24, 24, 27, 0.92)");
            ctx.fill_rect(w / 2.0 - 150.0, h - 45.0, 300.0, 32.0);
            ctx.set_stroke_style_str("#facc15");
            ctx.set_line_width(1.5);
            ctx.stroke_rect(w / 2.0 - 150.0, h - 45.0, 300.0, 32.0);

            ctx.set_font("bold 12px sans-serif");
            ctx.set_text_align("center");
            ctx.set_fill_style_str("#facc15");
            let _ = ctx.fill_text(&text, w / 2.0, h - 25.0);
        }

        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn draw_tent(ctx: &CanvasRenderingContext2d, x: f64, y: f64, color: &str) {
    ctx.set_fill_style_str(color);
    ctx.begin_path();
    ctx.move_to(x - 24.0, y + 18.0);
    ctx.line_to(x, y - 18.0);
    ctx.line_to(x + 24.0, y + 18.0);
    ctx.fill();
    ctx.set_fill_style_str("#78350f");
    ctx.begin_path();
    ctx.move_to(x, y - 18.0);
    ctx.line_to(x - 6.0, y + 18.0);
    ctx.line_to(x + 6.0, y + 18.0);
    ctx.fill();
}

fn draw_pillar(ctx: &CanvasRenderingContext2d, x: f64, y: f64, h: f64) {
    ctx.set_fill_style_str("#b45309"); // Cedarwood column
    ctx.fill_rect(x - 8.0, y, 16.0, h);
    ctx.set_fill_style_str("#fde047"); // Gold Capital
    ctx.fill_rect(x - 12.0, y - 8.0, 24.0, 10.0);
    ctx.fill_rect(x - 12.0, y + h - 2.0, 24.0, 10.0);
}

fn draw_medieval_house(ctx: &CanvasRenderingContext2d, x: f64, y: f64, color: &str) {
    ctx.set_fill_style_str(color);
    ctx.begin_path();
    ctx.move_to(x - 30.0, y + 10.0);
    ctx.line_to(x, y - 20.0);
    ctx.line_to(x + 30.0, y + 10.0);
    ctx.fill();
    ctx.set_fill_style_str("#fed7aa");
    ctx.fill_rect(x - 24.0, y + 10.0, 48.0, 36.0);
    ctx.set_fill_style_str("#0284c7");
    ctx.fill_rect(x - 14.0, y + 16.0, 10.0, 12.0);
    ctx.fill_rect(x + 4.0, y + 16.0, 10.0, 12.0);
}

fn draw_olive_tree(ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
    ctx.set_fill_style_str("#78350f");
    ctx.fill_rect(x - 4.0, y, 8.0, 26.0);
    ctx.set_fill_style_str("#15803d");
    ctx.begin_path();
    let _ = ctx.arc(x, y - 6.0, 18.0, 0.0, std::f64::consts::PI * 2.0);
    ctx.fill();
}

fn draw_pyramid(ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
    ctx.set_fill_style_str("#d97706");
    ctx.begin_path();
    ctx.move_to(x - 28.0, y + 25.0);
    ctx.line_to(x, y - 20.0);
    ctx.line_to(x + 28.0, y + 25.0);
    ctx.fill();
    ctx.set_fill_style_str("#b45309");
    ctx.begin_path();
    ctx.move_to(x, y - 20.0);
    ctx.line_to(x + 28.0, y + 25.0);
    ctx.line_to(x + 12.0, y + 25.0);
    ctx.fill();
}

fn draw_temple_facade(ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
    ctx.set_fill_style_str("#fef3c7");
    ctx.fill_rect(x - 30.0, y - 10.0, 60.0, 38.0);
    ctx.set_fill_style_str("#d97706");
    ctx.fill_rect(x - 34.0, y - 15.0, 68.0, 6.0);
    ctx.set_fill_style_str("#b45309");
    ctx.fill_rect(x - 22.0, y - 9.0, 5.0, 36.0);
    ctx.fill_rect(x - 3.0, y - 9.0, 5.0, 36.0);
    ctx.fill_rect(x + 16.0, y - 9.0, 5.0, 36.0);
}

fn draw_western_wall(ctx: &CanvasRenderingContext2d, x: f64, y: f64) {
    ctx.set_fill_style_str("#d6cbba");
    ctx.fill_rect(x - 40.0, y - 15.0, 80.0, 34.0);
    ctx.set_stroke_style_str("#9ca3af");
    ctx.set_line_width(1.0);
    ctx.stroke_rect(x - 40.0, y - 15.0, 80.0, 34.0);
    ctx.set_fill_style_str("#15803d");
    ctx.fill_rect(x - 10.0, y - 5.0, 8.0, 4.0);
    ctx.fill_rect(x + 14.0, y + 6.0, 10.0, 4.0);
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}
