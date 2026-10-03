use leptos::prelude::*;
use leptos_router::components::A;
use crate::simulator::models::{PlayerClass, ClientMessage};

#[component]
pub fn SimulatorPage() -> impl IntoView {
    let players = RwSignal::new(Vec::<crate::simulator::models::PlayerState>::new());
    let current_question = RwSignal::new(None::<String>);
    let current_scene = RwSignal::new("overview".to_string());
    let current_class = RwSignal::new(PlayerClass::Explorer);
    let is_muted = RwSignal::new(false);
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    #[cfg(feature = "hydrate")]
    {
        canvas_ref.on_load(move |canvas| {
            use wasm_bindgen::JsCast;
            let canvas_el: web_sys::HtmlCanvasElement = (*canvas).clone().dyn_into().unwrap();
            crate::simulator::client::init_game(players, current_question, current_scene, canvas_el);
        });

        on_cleanup(move || {
            crate::simulator::client::WS_INSTANCE.with(|ws_ref| {
                if let Some(ws) = ws_ref.borrow().as_ref() {
                    let _ = ws.close();
                }
                *ws_ref.borrow_mut() = None;
            });
        });
    }

    let toggle_mute = move |_| {
        let new_state = !is_muted.get();
        is_muted.set(new_state);
        #[cfg(feature = "hydrate")]
        crate::simulator::client::SOUND_MUTED.with(|m| {
            *m.borrow_mut() = new_state;
        });
    };

    let switch_scene = move |target: &'static str, tx: f64, ty: f64| {
        current_scene.set(target.to_string());
        #[cfg(feature = "hydrate")]
        {
            crate::simulator::client::play_sound("portal");
            crate::simulator::client::send_client_msg(ClientMessage::ChangeScene {
                scene: target.to_string(),
                x: tx,
                y: ty,
            });
        }
    };

    let select_class = move |c: PlayerClass| {
        current_class.set(c.clone());
        #[cfg(feature = "hydrate")]
        crate::simulator::client::send_client_msg(ClientMessage::SetClass(c));
    };

    view! {
        <div class="simulator-page" style="width: 100vw; height: 100vh; display: flex; flex-direction: column; background: #0c0a09; color: #fff; position: relative; user-select: none;">
            // Navigation Header
            <header style="padding: 0.6rem 1.2rem; background: #141210; border-bottom: 1px solid #292524; display: flex; justify-content: space-between; align-items: center; z-index: 10;">
                <div style="display: flex; align-items: center; gap: 12px;">
                    <A href="/" attr:style="color: #f59e0b; text-decoration: none; font-size: 13px; font-weight: bold; background: #292524; padding: 5px 10px; border-radius: 6px;">"← 返回主站"</A>
                    <span style="font-weight: bold; font-size: 15px; color: #fef3c7;">"🗺️ 历史漫游纪元"</span>
                </div>

                // Chapter Navigation (Fast Travel)
                <div style="display: flex; align-items: center; gap: 6px; background: #1c1917; padding: 4px 8px; border-radius: 8px; border: 1px solid #44403c;">
                    <button
                        on:click=move |_| switch_scene("overview", 420.0, 230.0)
                        style=move || format!("padding: 4px 10px; font-size: 12px; border-radius: 5px; border: none; cursor: pointer; font-weight: bold; background: {}; color: {};",
                            if current_scene.get() == "overview" { "#d97706" } else { "transparent" },
                            if current_scene.get() == "overview" { "#fff" } else { "#a8a29e" }
                        )
                    >
                        "🗺️ 总览大地图"
                    </button>
                    <button
                        on:click=move |_| switch_scene("sinai", 200.0, 220.0)
                        style=move || format!("padding: 4px 10px; font-size: 12px; border-radius: 5px; border: none; cursor: pointer; font-weight: bold; background: {}; color: {};",
                            if current_scene.get() == "sinai" { "#d97706" } else { "transparent" },
                            if current_scene.get() == "sinai" { "#fff" } else { "#a8a29e" }
                        )
                    >
                        "🇪🇬 1.西奈圣山营地"
                    </button>
                    <button
                        on:click=move |_| switch_scene("temple", 420.0, 300.0)
                        style=move || format!("padding: 4px 10px; font-size: 12px; border-radius: 5px; border: none; cursor: pointer; font-weight: bold; background: {}; color: {};",
                            if current_scene.get() == "temple" { "#d97706" } else { "transparent" },
                            if current_scene.get() == "temple" { "#fff" } else { "#a8a29e" }
                        )
                    >
                        "🏛️ 2.所罗门圣殿"
                    </button>
                    <button
                        on:click=move |_| switch_scene("europe", 420.0, 220.0)
                        style=move || format!("padding: 4px 10px; font-size: 12px; border-radius: 5px; border: none; cursor: pointer; font-weight: bold; background: {}; color: {};",
                            if current_scene.get() == "europe" { "#d97706" } else { "transparent" },
                            if current_scene.get() == "europe" { "#fff" } else { "#a8a29e" }
                        )
                    >
                        "🏰 3.欧洲中世纪城镇"
                    </button>
                    <button
                        on:click=move |_| switch_scene("peace", 420.0, 250.0)
                        style=move || format!("padding: 4px 10px; font-size: 12px; border-radius: 5px; border: none; cursor: pointer; font-weight: bold; background: {}; color: {};",
                            if current_scene.get() == "peace" { "#d97706" } else { "transparent" },
                            if current_scene.get() == "peace" { "#fff" } else { "#a8a29e" }
                        )
                    >
                        "🕊️ 4.和平讲坛"
                    </button>
                </div>

                // Right Utility Bar (Class, Mute, Online count)
                <div style="display: flex; align-items: center; gap: 14px;">
                    // Audio toggle
                    <button
                        on:click=toggle_mute
                        style="padding: 4px 8px; font-size: 12px; background: #292524; color: #fef3c7; border: 1px solid #44403c; border-radius: 6px; cursor: pointer;"
                    >
                        {move || if is_muted.get() { "🔇 静音" } else { "🔊 声音开" }}
                    </button>

                    // Online counter
                    <div style="display: flex; align-items: center; gap: 6px; background: #1c1917; padding: 4px 12px; border-radius: 16px; border: 1px solid #44403c;">
                        <span style="display: inline-block; width: 7px; height: 7px; border-radius: 50%; background: #22c55e;"></span>
                        <span style="color: #a8a29e; font-size: 12px;">"在线: "</span>
                        <span style="font-weight: bold; color: #22c55e; font-size: 13px;">{move || players.get().len()}</span>
                    </div>
                </div>
            </header>
            
            // Sub-bar: Class Selection & Perks
            <div style="padding: 0.4rem 1.5rem; background: #1c1917; border-bottom: 1px solid #292524; display: flex; justify-content: space-between; align-items: center; font-size: 12px;">
                <div style="display: flex; align-items: center; gap: 8px;">
                    <span style="color: #a8a29e;">"职业选择:"</span>
                    <button
                        on:click=move |_| select_class(PlayerClass::Explorer)
                        style=move || format!("padding: 2px 8px; border-radius: 4px; border: 1px solid #57534e; cursor: pointer; background: {}; color: {};",
                            if current_class.get() == PlayerClass::Explorer { "#d97706" } else { "#292524" },
                            if current_class.get() == PlayerClass::Explorer { "#fff" } else { "#d6d3d1" }
                        )
                    >
                        "🧭 探索者 (速度+35% & 宽阔视野)"
                    </button>
                    <button
                        on:click=move |_| select_class(PlayerClass::Scholar)
                        style=move || format!("padding: 2px 8px; border-radius: 4px; border: 1px solid #57534e; cursor: pointer; background: {}; color: {};",
                            if current_class.get() == PlayerClass::Scholar { "#d97706" } else { "#292524" },
                            if current_class.get() == PlayerClass::Scholar { "#fff" } else { "#d6d3d1" }
                        )
                    >
                        "📜 学者 (历史试炼经验 +50%)"
                    </button>
                    <button
                        on:click=move |_| select_class(PlayerClass::Merchant)
                        style=move || format!("padding: 2px 8px; border-radius: 4px; border: 1px solid #57534e; cursor: pointer; background: {}; color: {};",
                            if current_class.get() == PlayerClass::Merchant { "#d97706" } else { "#292524" },
                            if current_class.get() == PlayerClass::Merchant { "#fff" } else { "#d6d3d1" }
                        )
                    >
                        "🧮 商人 (信用账册加成)"
                    </button>
                </div>
                <div style="color: #facc15;">
                    {move || format!("当前加成: {}", current_class.get().perk())}
                </div>
            </div>

            // Main Canvas Area
            <main style="flex: 1; position: relative; display: flex; flex-direction: column; justify-content: center; align-items: center; overflow: hidden; background: #0c0a09;">
                <div style="position: relative; border-radius: 12px; overflow: hidden; box-shadow: 0 25px 60px rgba(0,0,0,0.85); border: 2px solid #44403c;">
                    // The HTML5 Game Canvas
                    <canvas node_ref=canvas_ref width="840" height="460" style="display: block; background: #1c1917;"></canvas>
                    
                    // Reactive DOM Overlay for Player Nametags
                    <div id="game-ui-layer" style="position: absolute; top: 0; left: 0; width: 840px; height: 460px; pointer-events: none;">
                        {move || {
                            let cur_sc = current_scene.get();
                            players.get().into_iter()
                                .filter(|p| p.scene == cur_sc)
                                .map(|p| {
                                    let label = format!("{} (Lv.{})", p.class.name(), p.level);
                                    view! {
                                        <div style=format!("position: absolute; left: {}px; top: {}px; transform: translate(-50%, -100%); margin-top: -24px; font-size: 11px; font-weight: bold; white-space: nowrap; pointer-events: none;", p.x, p.y)>
                                            <span style="background: rgba(12, 10, 9, 0.88); color: #fef08a; padding: 2px 8px; border-radius: 10px; border: 1px solid rgba(253, 224, 71, 0.5); box-shadow: 0 2px 6px rgba(0,0,0,0.6);">
                                                {label}
                                            </span>
                                        </div>
                                    }
                                }).collect_view()
                        }}
                    </div>
                </div>

                // Bottom Controls Legend
                <div style="margin-top: 12px; display: flex; gap: 20px; color: #a8a29e; font-size: 12px;">
                    <span>"⌨️ " <strong style="color: #f3f4f6;">"W/A/S/D"</strong> " 或 " <strong style="color: #f3f4f6;">"方向键"</strong> " 平滑移动"</span>
                    <span>"🌀 踩入地图传送阵 或 顶部章节栏 可在各大历史篇章中自如穿梭"</span>
                    <span>"💬 靠近历史向导/遗迹 按 " <strong style="color: #facc15;">"[E]"</strong> " 或 " <strong style="color: #facc15;">"[空格]"</strong> " 答题升级"</span>
                </div>

                // Modal for Historical Trials
                <Show when=move || current_question.get().is_some() fallback=|| ()>
                    <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.8); backdrop-filter: blur(5px); display: flex; justify-content: center; align-items: center; z-index: 50;">
                        <div style="background: #1c1917; border: 2px solid #d97706; border-radius: 16px; padding: 28px; max-width: 580px; width: 90%; color: #f5f5f4; box-shadow: 0 30px 70px rgba(0,0,0,0.9);">
                            <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #44403c; padding-bottom: 12px; margin-bottom: 16px;">
                                <h3 style="margin: 0; font-size: 18px; color: #fbbf24; display: flex; align-items: center; gap: 8px;">
                                    "📖 历史知识试炼与传承"
                                </h3>
                                <span style="font-size: 12px; background: #78350f; color: #fde68a; padding: 3px 10px; border-radius: 12px; font-weight: bold;">
                                    {move || if current_class.get() == PlayerClass::Scholar { "+40 学者智慧经验" } else { "+25 智慧经验" }}
                                </span>
                            </div>

                            <div style="font-size: 14.5px; line-height: 1.7; color: #e7e5e4; white-space: pre-wrap; margin-bottom: 24px;">
                                {move || current_question.get().unwrap_or_default()}
                            </div>

                            <div style="display: flex; justify-content: flex-end; gap: 14px;">
                                <button
                                    on:click=move |_| current_question.set(None)
                                    style="padding: 9px 18px; background: #292524; color: #d6d3d1; border: 1px solid #57534e; border-radius: 8px; font-size: 14px; cursor: pointer;"
                                >
                                    "稍后思考"
                                </button>
                                <button
                                    on:click=move |_| {
                                        #[cfg(feature = "hydrate")]
                                        {
                                            crate::simulator::client::play_sound("victory");
                                            crate::simulator::client::send_client_msg(ClientMessage::LevelUp);
                                        }
                                        current_question.set(None);
                                    }
                                    style="padding: 9px 20px; background: #d97706; color: #1c1917; font-weight: bold; border: none; border-radius: 8px; font-size: 14px; cursor: pointer; box-shadow: 0 4px 12px rgba(217, 119, 6, 0.4);"
                                >
                                    "🌟 铭记领悟 (+升级)"
                                </button>
                            </div>
                        </div>
                    </div>
                </Show>
            </main>
        </div>
    }
}
