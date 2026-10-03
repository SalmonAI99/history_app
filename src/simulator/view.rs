use leptos::prelude::*;
use leptos_router::components::A;
use crate::simulator::models::PlayerClass;
#[cfg(feature = "hydrate")]
use crate::simulator::models::ClientMessage;

#[component]
pub fn SimulatorPage() -> impl IntoView {
    let players = RwSignal::new(Vec::<crate::simulator::models::PlayerState>::new());
    let current_question = RwSignal::new(None::<String>);
    let current_scene = RwSignal::new("overview".to_string());
    let current_class = RwSignal::new(PlayerClass::Explorer);
    let view_mode = RwSignal::new("3d".to_string()); // "3d" (World Simulator Sandbox) or "2d" (Historical RPG)
    let current_3d_mode = RwSignal::new("caravan".to_string()); // "caravan", "temple_city", "scribes", "wisdom"
    let is_muted = RwSignal::new(false);
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    // 2D Game initialization
    #[cfg(feature = "hydrate")]
    {
        use std::rc::Rc;
        use std::cell::Cell;
        use wasm_bindgen::JsCast;

        let has_started = Rc::new(Cell::new(false));

        let start_fn = {
            let has_started = has_started.clone();
            move |el: web_sys::HtmlCanvasElement| {
                if has_started.get() { return; }
                has_started.set(true);
                crate::simulator::client::init_game(players, current_question, current_scene, el);
            }
        };

        let sf1 = start_fn.clone();
        canvas_ref.on_load(move |canvas| {
            let canvas_el: web_sys::HtmlCanvasElement = (*canvas).clone().dyn_into().unwrap();
            sf1(canvas_el);
        });

        let sf2 = start_fn.clone();
        Effect::new(move |_| {
            if has_started.get() { return; }
            if let Some(canvas) = canvas_ref.get() {
                let canvas_el: web_sys::HtmlCanvasElement = (*canvas).clone().dyn_into().unwrap();
                sf2(canvas_el);
            } else if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(el) = doc.get_element_by_id("game-canvas") {
                    if let Ok(canvas_el) = el.dyn_into::<web_sys::HtmlCanvasElement>() {
                        sf2(canvas_el);
                    }
                }
            }
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

    // 3D Three.js Sandbox script loader & initializer
    let init_3d_sandbox = move || {
        #[cfg(feature = "hydrate")]
        {
            let script = r#"
            (function() {
                function loadScript(src, cb) {
                    var existing = document.querySelector('script[src="' + src + '"]');
                    if (existing) { cb(); return; }
                    var s = document.createElement('script');
                    s.src = src;
                    s.onload = cb;
                    document.head.appendChild(s);
                }
                loadScript('/three.min.js', function() {
                    loadScript('/OrbitControls.js', function() {
                        loadScript('/GLTFLoader.js', function() {
                            loadScript('/SkeletonUtils.js', function() {
                                loadScript('/simulator/sandbox.js', function() {
                                    var container = document.getElementById('sandbox-3d-viewport');
                                    if (container && window.initSandbox3D) {
                                        window.initSandbox3D(container);
                                    }
                                });
                            });
                        });
                    });
                });
            })();
            "#;
            let _ = js_sys::eval(script);
        }
    };

    // Trigger 3D initialization when switching to 3d mode
    Effect::new(move |_| {
        if view_mode.get() == "3d" {
            init_3d_sandbox();
        }
    });

    let toggle_mute = move |_| {
        let new_state = !is_muted.get();
        is_muted.set(new_state);
        #[cfg(feature = "hydrate")]
        crate::simulator::client::SOUND_MUTED.with(|m| {
            *m.borrow_mut() = new_state;
        });
    };

    let switch_scene = move |target: &'static str, #[allow(unused_variables)] tx: f64, #[allow(unused_variables)] ty: f64| {
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

    let _select_class = move |#[allow(unused_variables)] c: PlayerClass| {
        #[cfg(feature = "hydrate")]
        {
            current_class.set(c.clone());
            crate::simulator::client::send_client_msg(ClientMessage::SetClass(c));
        }
    };

    let set_3d_mode = move |m: &'static str| {
        current_3d_mode.set(m.to_string());
        #[cfg(feature = "hydrate")]
        {
            let code = format!("if (window.setSimulationMode) window.setSimulationMode('{}');", m);
            let _ = js_sys::eval(&code);
        }
    };

    let set_3d_speed = move |#[allow(unused_variables)] s: f64| {
        #[cfg(feature = "hydrate")]
        {
            let code = format!("if (window.setSandboxSpeed) window.setSandboxSpeed({});", s);
            let _ = js_sys::eval(&code);
        }
    };

    let toggle_3d_play = move |_| {
        #[cfg(feature = "hydrate")]
        {
            let _ = js_sys::eval("if (window.toggleSandboxPlayback) window.toggleSandboxPlayback();");
        }
    };

    let start_control_roam = move |_| {
        #[cfg(feature = "hydrate")]
        {
            let _ = js_sys::eval("if (window.controlRandomCitizen) window.controlRandomCitizen();");
        }
    };

    let start_control_knight = move |_| {
        #[cfg(feature = "hydrate")]
        {
            let _ = js_sys::eval("if (window.takeControlOfKnight) window.takeControlOfKnight();");
        }
    };

    let focus_landmark = move |name: &'static str| {
        #[cfg(feature = "hydrate")]
        {
            let code = format!("if (window.focusLandmark) window.focusLandmark('{}');", name);
            let _ = js_sys::eval(&code);
        }
    };

    let set_time_of_day = move |tod: &'static str| {
        #[cfg(feature = "hydrate")]
        {
            let code = format!("if (window.setTimeOfDay) window.setTimeOfDay('{}');", tod);
            let _ = js_sys::eval(&code);
        }
    };

    let trigger_event = move |evt: &'static str| {
        #[cfg(feature = "hydrate")]
        {
            let code = format!("if (window.triggerHistoricalEvent) window.triggerHistoricalEvent('{}');", evt);
            let _ = js_sys::eval(&code);
        }
    };

    view! {
        <div class="simulator-page" style="width: 100vw; height: 100vh; display: flex; flex-direction: column; background: #0b1220; color: #fff; position: relative; user-select: none; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;">
            // Navigation Header
            <header style="padding: 0.6rem 1.4rem; background: #0f172a; border-bottom: 1px solid #1e293b; display: flex; justify-content: space-between; align-items: center; z-index: 10;">
                <div style="display: flex; align-items: center; gap: 14px;">
                    <A href="/" attr:style="color: #38bdf8; text-decoration: none; font-size: 13px; font-weight: bold; background: #1e293b; padding: 5px 12px; border-radius: 6px;">"← 返回主站"</A>
                    <span style="font-weight: 800; font-size: 16px; color: #f8fafc; letter-spacing: -0.02em;">"WORLD SIMULATOR"</span>
                </div>

                // Primary Mode Toggle: 3D Multi-Agent Sandbox vs 2D Historical RPG
                <div style="display: flex; align-items: center; gap: 6px; background: #020617; padding: 4px; border-radius: 8px; border: 1px solid #1e293b;">
                    <button
                        on:click=move |_| view_mode.set("3d".to_string())
                        style=move || format!("padding: 5px 14px; font-size: 13px; font-weight: bold; border-radius: 6px; border: none; cursor: pointer; transition: all 0.2s; background: {}; color: {};",
                            if view_mode.get() == "3d" { "#2563eb" } else { "transparent" },
                            if view_mode.get() == "3d" { "#fff" } else { "#94a3b8" }
                        )
                    >
                        "🌐 3D 智能体沙盒 (World Simulator)"
                    </button>
                    <button
                        on:click=move |_| view_mode.set("2d".to_string())
                        style=move || format!("padding: 5px 14px; font-size: 13px; font-weight: bold; border-radius: 6px; border: none; cursor: pointer; transition: all 0.2s; background: {}; color: {};",
                            if view_mode.get() == "2d" { "#d97706" } else { "transparent" },
                            if view_mode.get() == "2d" { "#fff" } else { "#94a3b8" }
                        )
                    >
                        "🗺️ 2D 历史漫游 RPG"
                    </button>
                </div>

                // Right Utility Bar
                <div style="display: flex; align-items: center; gap: 12px;">
                    <button
                        on:click=toggle_mute
                        style="padding: 5px 10px; font-size: 12px; background: #1e293b; color: #f8fafc; border: 1px solid #334155; border-radius: 6px; cursor: pointer;"
                    >
                        {move || if is_muted.get() { "🔇 静音" } else { "🔊 声音开" }}
                    </button>

                    <div style="display: flex; align-items: center; gap: 6px; background: #1e293b; padding: 5px 12px; border-radius: 16px; border: 1px solid #334155;">
                        <span style="display: inline-block; width: 8px; height: 8px; border-radius: 50%; background: #22c55e;"></span>
                        <span style="color: #94a3b8; font-size: 12px;">"在线: "</span>
                        <span style="font-weight: bold; color: #22c55e; font-size: 13px;">{move || players.get().len()}</span>
                    </div>
                </div>
            </header>
            
            // ==========================================
            // VIEW MODE 1: 3D MULTI-AGENT SIMULATION SANDBOX
            // ==========================================
            <Show when=move || view_mode.get() == "3d" fallback=|| ()>
                <div style="flex: 1; display: flex; flex-direction: column; position: relative; background: #0b1220; overflow: hidden;">
                    // 3D Sub-Header: Mode Selector & Telemetry
                    <div style="padding: 0.6rem 1.4rem; background: #0f172a; border-bottom: 1px solid #1e293b; display: flex; justify-content: space-between; align-items: center;">
                        <div style="display: flex; gap: 8px;">
                            <button
                                on:click=move |_| set_3d_mode("caravan")
                                style=move || format!(
                                    "padding: 5px 12px; font-size: 12px; font-weight: bold; border-radius: 6px; cursor: pointer; transition: all 0.2s; background: {}; color: #38bdf8; border: 1px solid {};",
                                    if current_3d_mode.get() == "caravan" { "#0369a1" } else { "#1e293b" },
                                    if current_3d_mode.get() == "caravan" { "#38bdf8" } else { "#0284c7" }
                                )
                            >
                                "🐪 古代商道与驼队 (IDM物流)"
                            </button>
                            <button
                                on:click=move |_| set_3d_mode("temple_city")
                                style=move || format!(
                                    "padding: 5px 12px; font-size: 12px; font-weight: bold; border-radius: 6px; cursor: pointer; transition: all 0.2s; background: {}; color: #4ade80; border: 1px solid {};",
                                    if current_3d_mode.get() == "temple_city" { "#15803d" } else { "#1e293b" },
                                    if current_3d_mode.get() == "temple_city" { "#4ade80" } else { "#16a34a" }
                                )
                            >
                                "🏛️ 圣殿城邦与繁荣 (城邦ABM)"
                            </button>
                            <button
                                on:click=move |_| set_3d_mode("scribes")
                                style=move || format!(
                                    "padding: 5px 12px; font-size: 12px; font-weight: bold; border-radius: 6px; cursor: pointer; transition: all 0.2s; background: {}; color: #fbbf24; border: 1px solid {};",
                                    if current_3d_mode.get() == "scribes" { "#b45309" } else { "#1e293b" },
                                    if current_3d_mode.get() == "scribes" { "#fbbf24" } else { "#d97706" }
                                )
                            >
                                "📜 旷野迁徙与抄经使 (集群协作)"
                            </button>
                            <button
                                on:click=move |_| set_3d_mode("wisdom")
                                style=move || format!(
                                    "padding: 5px 12px; font-size: 12px; font-weight: bold; border-radius: 6px; cursor: pointer; transition: all 0.2s; background: {}; color: #c084fc; border: 1px solid {};",
                                    if current_3d_mode.get() == "wisdom" { "#7e22ce" } else { "#1e293b" },
                                    if current_3d_mode.get() == "wisdom" { "#c084fc" } else { "#9333ea" }
                                )
                            >
                                "🕊️ 智者论辩与思想派系 (回音室图谱)"
                            </button>
                        </div>

                        // Speed and Playback Controls
                        <div style="display: flex; align-items: center; gap: 10px;">
                            <button
                                on:click=toggle_3d_play
                                style="padding: 5px 14px; background: #2563eb; color: #fff; font-weight: bold; font-size: 12px; border: none; border-radius: 6px; cursor: pointer;"
                            >
                                "⏯️ 播放/暂停"
                            </button>
                            <div style="display: flex; background: #1e293b; padding: 2px; border-radius: 6px; border: 1px solid #334155;">
                                <button on:click=move |_| set_3d_speed(1.0) style="padding: 3px 8px; font-size: 11px; background: transparent; color: #e2e8f0; border: none; cursor: pointer;">"1x"</button>
                                <button on:click=move |_| set_3d_speed(2.0) style="padding: 3px 8px; font-size: 11px; background: transparent; color: #e2e8f0; border: none; cursor: pointer;">"2x"</button>
                                <button on:click=move |_| set_3d_speed(5.0) style="padding: 3px 8px; font-size: 11px; background: transparent; color: #e2e8f0; border: none; cursor: pointer;">"5x"</button>
                            </div>

                            <button
                                on:click=start_control_roam
                                style="padding: 5px 12px; background: linear-gradient(135deg, #10b981, #059669); color: #fff; font-weight: bold; font-size: 12px; border: none; border-radius: 6px; cursor: pointer; display: flex; align-items: center; gap: 5px; box-shadow: 0 2px 8px rgba(16, 185, 129, 0.4);"
                            >
                                "🎮 附身平民"
                            </button>
                            <button
                                on:click=start_control_knight
                                style="padding: 5px 12px; background: linear-gradient(135deg, #0284c7, #1d4ed8); color: #fff; font-weight: bold; font-size: 12px; border: none; border-radius: 6px; cursor: pointer; display: flex; align-items: center; gap: 5px; box-shadow: 0 2px 8px rgba(2, 132, 199, 0.4);"
                            >
                                "⚔️ 附身圣殿骑士 (WASD)"
                            </button>

                            <div style="display: flex; gap: 8px; margin-left: 10px; font-family: monospace; font-size: 12px;">
                                <span style="background: #1e293b; padding: 4px 8px; border-radius: 4px; color: #38bdf8;" id="sb-fps">"60 FPS"</span>
                                <span style="background: #1e293b; padding: 4px 8px; border-radius: 4px; color: #4ade80;"><span id="sb-agents-label">"活跃个体: "</span><strong id="sb-agents">"35"</strong></span>
                            </div>
                        </div>
                    </div>

                    // 3D Canvas Viewport
                    <div id="sandbox-3d-viewport" style="flex: 1; width: 100%; height: 100%; position: relative; overflow: hidden;">
                        // Dedicated unmanaged 3D canvas container to prevent hydration interference
                        <div id="sandbox-canvas-container" style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; z-index: 1;"></div>

                        // 1. Top Center Floating Landmark Tour Bar
                        <div style="position: absolute; top: 12px; left: 50%; transform: translateX(-50%); z-index: 25; display: flex; gap: 5px; background: rgba(15, 23, 42, 0.92); backdrop-filter: blur(8px); padding: 5px 12px; border-radius: 20px; border: 1px solid #334155; box-shadow: 0 8px 24px rgba(0,0,0,0.5); max-width: 95vw; overflow-x: auto;">
                            <button on:click=start_control_knight style="padding: 3px 10px; font-size: 11px; background: linear-gradient(135deg, #0284c7, #1d4ed8); color: #fff; font-weight: 800; border: 1px solid #38bdf8; border-radius: 12px; cursor: pointer; white-space: nowrap; box-shadow: 0 0 10px rgba(2, 132, 199, 0.4);">"⚔️ 附身圣殿骑士 (WASD)"</button>
                            <button on:click=start_control_roam style="padding: 3px 8px; font-size: 11px; background: #1e293b; color: #6ee7b7; border: 1px solid #059669; border-radius: 12px; cursor: pointer; white-space: nowrap;">"👥 随机平民"</button>
                            <span style="font-size: 11px; font-weight: bold; color: #94a3b8; align-self: center; margin: 0 2px; white-space: nowrap;">"|"</span>
                            <span style="font-size: 11px; font-weight: bold; color: #94a3b8; align-self: center; margin-right: 4px; white-space: nowrap;">"🏛️ 地标巡航:"</span>
                            <button on:click=move |_| focus_landmark("temple_solomon") style="padding: 3px 8px; font-size: 11px; background: #1e293b; color: #fde047; border: 1px solid #78350f; border-radius: 12px; cursor: pointer; white-space: nowrap;">"第一圣殿"</button>
                            <button on:click=move |_| focus_landmark("palace_lebanon") style="padding: 3px 8px; font-size: 11px; background: #1e293b; color: #c084fc; border: 1px solid #581c87; border-radius: 12px; cursor: pointer; white-space: nowrap;">"林宫王廷"</button>
                            <button on:click=move |_| focus_landmark("royal_granary") style="padding: 3px 8px; font-size: 11px; background: #1e293b; color: #fbbf24; border: 1px solid #78350f; border-radius: 12px; cursor: pointer; white-space: nowrap;">"战略粮仓"</button>
                            <button on:click=move |_| focus_landmark("gihon_cistern") style="padding: 3px 8px; font-size: 11px; background: #1e293b; color: #38bdf8; border: 1px solid #0369a1; border-radius: 12px; cursor: pointer; white-space: nowrap;">"基训水池"</button>
                            <button on:click=move |_| focus_landmark("central_bazaar") style="padding: 3px 8px; font-size: 11px; background: #1e293b; color: #f87171; border: 1px solid #991b1b; border-radius: 12px; cursor: pointer; white-space: nowrap;">"彩篷集市"</button>
                            <button on:click=move |_| focus_landmark("bronze_foundry") style="padding: 3px 8px; font-size: 11px; background: #1e293b; color: #fb923c; border: 1px solid #9a3412; border-radius: 12px; cursor: pointer; white-space: nowrap;">"青铜工坊"</button>
                            <button on:click=move |_| focus_landmark("olive_press") style="padding: 3px 8px; font-size: 11px; background: #1e293b; color: #4ade80; border: 1px solid #166534; border-radius: 12px; cursor: pointer; white-space: nowrap;">"橄榄石磨"</button>
                            <button on:click=move |_| focus_landmark("four_room_house") style="padding: 3px 8px; font-size: 11px; background: #1e293b; color: #fed7aa; border: 1px solid #7c2d12; border-radius: 12px; cursor: pointer; white-space: nowrap;">"四室民居"</button>
                            <button on:click=move |_| focus_landmark("bush_flora") style="padding: 3px 8px; font-size: 11px; background: #1e293b; color: #86efac; border: 1px solid #16a34a; border-radius: 12px; cursor: pointer; white-space: nowrap;">"🌿 圣木/灌木"</button>
                            <button on:click=move |_| focus_landmark("city_gate_elders") style="padding: 3px 8px; font-size: 11px; background: #1e293b; color: #e2e8f0; border: 1px solid #475569; border-radius: 12px; cursor: pointer; white-space: nowrap;">"六室城门"</button>
                        </div>

                        // 2. Top Left Floating Environment & Historical Events Bar
                        <div style="position: absolute; top: 12px; left: 16px; z-index: 25; display: flex; flex-direction: column; gap: 6px;">
                            // Day/Night Sky Controller
                            <div style="display: flex; gap: 3px; background: rgba(15, 23, 42, 0.92); backdrop-filter: blur(6px); padding: 3px 6px; border-radius: 8px; border: 1px solid #334155;">
                                <button on:click=move |_| set_time_of_day("dawn") style="padding: 3px 6px; font-size: 11px; background: transparent; color: #fda4af; border: none; cursor: pointer;">"🌅 黎明"</button>
                                <button on:click=move |_| set_time_of_day("noon") style="padding: 3px 6px; font-size: 11px; background: transparent; color: #fef08a; border: none; cursor: pointer;">"☀️ 正午"</button>
                                <button on:click=move |_| set_time_of_day("dusk") style="padding: 3px 6px; font-size: 11px; background: transparent; color: #fdba74; border: none; cursor: pointer;">"🌇 黄昏"</button>
                                <button on:click=move |_| set_time_of_day("night") style="padding: 3px 6px; font-size: 11px; background: transparent; color: #93c5fd; border: none; cursor: pointer;">"🌙 星夜"</button>
                            </div>

                            // Historical Event Triggers
                            <div style="display: flex; gap: 4px; background: rgba(15, 23, 42, 0.92); backdrop-filter: blur(6px); padding: 3px 6px; border-radius: 8px; border: 1px solid #334155;">
                                <button on:click=move |_| trigger_event("caravan_arrival") style="padding: 3px 7px; font-size: 11px; background: #1e293b; color: #f59e0b; border: 1px solid #d97706; border-radius: 5px; cursor: pointer;">"🐫 商队抵城"</button>
                                <button on:click=move |_| trigger_event("divine_rain") style="padding: 3px 7px; font-size: 11px; background: #1e293b; color: #38bdf8; border: 1px solid #0284c7; border-radius: 5px; cursor: pointer;">"🌧️ 祈降甘霖"</button>
                                <button on:click=move |_| trigger_event("bush_bloom") style="padding: 3px 7px; font-size: 11px; background: #1e293b; color: #86efac; border: 1px solid #16a34a; border-radius: 5px; cursor: pointer;">"🌿 圣木郁葱"</button>
                                <button on:click=move |_| trigger_event("jubilee_feast") style="padding: 3px 7px; font-size: 11px; background: #1e293b; color: #ec4899; border: 1px solid #db2777; border-radius: 5px; cursor: pointer;">"🕊️ 禧年节期"</button>
                                <button on:click=move |_| trigger_event("year_of_abundance") style="padding: 3px 7px; font-size: 11px; background: #1e293b; color: #4ade80; border: 1px solid #16a34a; border-radius: 5px; cursor: pointer;">"🌾 丰收之年"</button>
                            </div>
                        </div>

                        // 3. Real-time Selected Agent / Building Inspector Overlay
                        <div id="sb-inspector" style="position: absolute; right: 20px; top: 12px; background: rgba(15, 23, 42, 0.95); backdrop-filter: blur(10px); border: 1px solid #475569; border-radius: 12px; padding: 16px; width: 320px; max-width: 90vw; max-height: 84vh; overflow-y: auto; box-shadow: 0 16px 36px rgba(0,0,0,0.75); display: none; z-index: 30;">
                            <div style="font-weight: bold; font-size: 14px; color: #38bdf8; margin-bottom: 8px;" id="sb-insp-title">"详情档案"</div>
                            <div style="font-size: 12px; color: #cbd5e1; line-height: 1.6;" id="sb-insp-body"></div>
                        </div>

                        // 4. Bottom Left Minimap Tactical Radar
                        <div style="position: absolute; left: 16px; bottom: 84px; z-index: 25; background: rgba(15, 23, 42, 0.92); backdrop-filter: blur(8px); border: 1px solid #334155; border-radius: 12px; padding: 6px; box-shadow: 0 10px 25px rgba(0,0,0,0.6); display: flex; flex-direction: column; align-items: center;">
                            <div style="font-size: 10px; font-weight: bold; color: #94a3b8; margin-bottom: 3px; letter-spacing: 0.05em;">"城邦战术雷达"</div>
                            <canvas id="sb-minimap" width="110" height="110" style="display: block; cursor: crosshair; border-radius: 50%;"></canvas>
                        </div>

                        // 5. Bottom Control Drawer for Sliders & Status
                        <div style="position: absolute; left: 16px; bottom: 16px; z-index: 20; background: rgba(15, 23, 42, 0.92); backdrop-filter: blur(8px); border: 1px solid #334155; border-radius: 10px; padding: 10px 16px; box-shadow: 0 10px 30px rgba(0,0,0,0.6);">
                            <div style="font-size: 11px; font-weight: bold; color: #94a3b8; text-transform: uppercase; margin-bottom: 6px;">"实时参数与城邦治理"</div>
                            <div id="sb-sliders" style="display: flex; gap: 20px; font-size: 12px; color: #e2e8f0; align-items: center;"></div>
                        </div>

                        // 6. Navigation Hint
                        <div style="position: absolute; right: 20px; bottom: 16px; pointer-events: none; font-size: 11px; color: #94a3b8; background: rgba(15, 23, 42, 0.75); backdrop-filter: blur(4px); padding: 5px 10px; border-radius: 6px; border: 1px solid #334155;">
                            "🖱️ 左键旋转 · 右键平移 · 滚轮缩放 · 悬停点击建筑或个体"
                        </div>
                    </div>
                </div>
            </Show>

            // ==========================================
            // VIEW MODE 2: 2D HISTORICAL ODYSSEY RPG
            // ==========================================
            <Show when=move || view_mode.get() == "2d" fallback=|| ()>
                <div style="flex: 1; display: flex; flex-direction: column; position: relative;">
                    // Chapter Navigation (Fast Travel)
                    <div style="padding: 0.4rem 1.4rem; background: #141210; border-bottom: 1px solid #292524; display: flex; justify-content: space-between; align-items: center;">
                        <div style="display: flex; align-items: center; gap: 6px;">
                            <button on:click=move |_| switch_scene("overview", 420.0, 230.0) style="padding: 4px 10px; font-size: 12px; border-radius: 5px; border: none; cursor: pointer; font-weight: bold; background: #292524; color: #fde68a;">"🗺️ 总览大地图"</button>
                            <button on:click=move |_| switch_scene("sinai", 200.0, 220.0) style="padding: 4px 10px; font-size: 12px; border-radius: 5px; border: none; cursor: pointer; font-weight: bold; background: #292524; color: #fde68a;">"🇪🇬 1.西奈圣山营地"</button>
                            <button on:click=move |_| switch_scene("temple", 420.0, 300.0) style="padding: 4px 10px; font-size: 12px; border-radius: 5px; border: none; cursor: pointer; font-weight: bold; background: #292524; color: #fde68a;">"🏛️ 2.所罗门圣殿"</button>
                            <button on:click=move |_| switch_scene("europe", 420.0, 220.0) style="padding: 4px 10px; font-size: 12px; border-radius: 5px; border: none; cursor: pointer; font-weight: bold; background: #292524; color: #fde68a;">"🏰 3.欧洲中世纪城镇"</button>
                            <button on:click=move |_| switch_scene("peace", 420.0, 250.0) style="padding: 4px 10px; font-size: 12px; border-radius: 5px; border: none; cursor: pointer; font-weight: bold; background: #292524; color: #fde68a;">"🕊️ 4.和平讲坛"</button>
                        </div>
                        <div style="font-size: 12px; color: #facc15;">
                            {move || format!("职业加成: {}", current_class.get().perk())}
                        </div>
                    </div>

                    // Main 2D Canvas Area
                    <main style="flex: 1; position: relative; display: flex; flex-direction: column; justify-content: center; align-items: center; overflow: hidden; background: #0c0a09;">
                        <div style="position: relative; border-radius: 12px; overflow: hidden; box-shadow: 0 25px 60px rgba(0,0,0,0.85); border: 2px solid #44403c;">
                            <canvas id="game-canvas" node_ref=canvas_ref width="840" height="460" style="display: block; background: #fde68a;"></canvas>
                            
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

                        <div style="margin-top: 12px; display: flex; gap: 20px; color: #a8a29e; font-size: 12px;">
                            <span>"⌨️ W/A/S/D 或 方向键 移动"</span>
                            <span>"🌀 踩入传送阵穿行各大历史篇章"</span>
                            <span>"💬 靠近人物按 [E] 对话答题升级"</span>
                        </div>
                    </main>
                </div>
            </Show>

            // Modal for Historical Trials (Works across modes)
            <Show when=move || current_question.get().is_some() fallback=|| ()>
                <div style="position: absolute; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.8); backdrop-filter: blur(5px); display: flex; justify-content: center; align-items: center; z-index: 50;">
                    <div style="background: #1c1917; border: 2px solid #d97706; border-radius: 16px; padding: 28px; max-width: 580px; width: 90%; color: #f5f5f4; box-shadow: 0 30px 70px rgba(0,0,0,0.9);">
                        <div style="display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #44403c; padding-bottom: 12px; margin-bottom: 16px;">
                            <h3 style="margin: 0; font-size: 18px; color: #fbbf24;">"📖 历史知识试炼与传承"</h3>
                            <span style="font-size: 12px; background: #78350f; color: #fde68a; padding: 3px 10px; border-radius: 12px; font-weight: bold;">
                                {move || if current_class.get() == PlayerClass::Scholar { "+40 学者智慧经验" } else { "+25 智慧经验" }}
                            </span>
                        </div>
                        <div style="font-size: 14.5px; line-height: 1.7; color: #e7e5e4; white-space: pre-wrap; margin-bottom: 24px;">
                            {move || current_question.get().unwrap_or_default()}
                        </div>
                        <div style="display: flex; justify-content: flex-end; gap: 14px;">
                            <button on:click=move |_| current_question.set(None) style="padding: 9px 18px; background: #292524; color: #d6d3d1; border: 1px solid #57534e; border-radius: 8px; font-size: 14px; cursor: pointer;">"稍后思考"</button>
                            <button
                                on:click=move |_| {
                                    #[cfg(feature = "hydrate")]
                                    {
                                        crate::simulator::client::play_sound("victory");
                                        crate::simulator::client::send_client_msg(ClientMessage::LevelUp);
                                    }
                                    current_question.set(None);
                                }
                                style="padding: 9px 20px; background: #d97706; color: #1c1917; font-weight: bold; border: none; border-radius: 8px; font-size: 14px; cursor: pointer;"
                            >
                                "🌟 铭记领悟 (+升级)"
                            </button>
                        </div>
                    </div>
                </div>
            </Show>
        </div>
    }
}
