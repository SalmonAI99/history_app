use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::{broadcast, RwLock};
use super::models::{ClientMessage, ServerMessage, PlayerState, PlayerClass};

pub struct GameState {
    pub tx: broadcast::Sender<ServerMessage>,
    pub players: RwLock<HashMap<String, PlayerState>>,
}

impl GameState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { 
            tx,
            players: RwLock::new(HashMap::new()),
        }
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    axum::Extension(state): axum::Extension<Arc<GameState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<GameState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.tx.subscribe();

    let player_id = format!("p-{}", uuid::Uuid::new_v4().to_string().chars().take(6).collect::<String>());

    let colors = ["#2563eb", "#d97706", "#059669", "#7c3aed", "#dc2626", "#0891b2"];
    let color_idx = rand_seed(&player_id) % colors.len();
    let player_color = colors[color_idx].to_string();

    let me = PlayerState {
        id: player_id.clone(),
        x: 100.0,
        y: 330.0,
        z: 0.0,
        ry: 0.0,
        anim: "idle".to_string(),
        model: "knight".to_string(),
        color: player_color,
        class: PlayerClass::Explorer,
        level: 1,
        knowledge: 0,
        scene: "overview".to_string(),
    };

    // Store the player
    {
        let mut players = state.players.write().await;
        players.insert(player_id.clone(), me.clone());
        
        let all_players: Vec<PlayerState> = players.values().cloned().collect();
        let init_msg = ServerMessage::StateSync { players: all_players };
        let welcome_msg = ServerMessage::Welcome { id: player_id.clone() };
        if let Ok(text) = serde_json::to_string(&init_msg) {
            let _ = sender.send(Message::Text(text.into())).await;
        }
        if let Ok(text) = serde_json::to_string(&welcome_msg) {
            let _ = sender.send(Message::Text(text.into())).await;
        }
    }

    let _ = state.tx.send(ServerMessage::PlayerJoined { player: me });

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(text) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    let state_clone = state.clone();
    let pid = player_id.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(text))) = receiver.next().await {
            if let Ok(msg) = serde_json::from_str::<ClientMessage>(&text) {
                match msg {
                    ClientMessage::Move { dx, dy } => {
                        let mut nx = 0.0;
                        let mut ny = 0.0;
                        if let Some(p) = state_clone.players.write().await.get_mut(&pid) {
                            p.x = (p.x + dx).clamp(30.0, 810.0);
                            p.y = (p.y + dy).clamp(30.0, 430.0);
                            nx = p.x;
                            ny = p.y;
                        }
                        let _ = state_clone.tx.send(ServerMessage::PlayerMoved { id: pid.clone(), x: nx, y: ny });
                    }
                    ClientMessage::SetClass(c) => {
                        let mut p_copy = None;
                        if let Some(p) = state_clone.players.write().await.get_mut(&pid) {
                            p.class = c;
                            p_copy = Some(p.clone());
                        }
                        if let Some(p) = p_copy {
                            let _ = state_clone.tx.send(ServerMessage::PlayerUpdated { player: p });
                        }
                    }
                    ClientMessage::ChangeScene { scene, x, y } => {
                        let mut p_copy = None;
                        if let Some(p) = state_clone.players.write().await.get_mut(&pid) {
                            p.scene = scene;
                            p.x = x;
                            p.y = y;
                            p_copy = Some(p.clone());
                        }
                        if let Some(p) = p_copy {
                            let _ = state_clone.tx.send(ServerMessage::PlayerUpdated { player: p });
                        }
                    }
                    ClientMessage::LevelUp => {
                        let mut p_copy = None;
                        if let Some(p) = state_clone.players.write().await.get_mut(&pid) {
                            let gain = if p.class == PlayerClass::Scholar { 40 } else { 25 };
                            p.level += 1;
                            p.knowledge += gain;
                            p_copy = Some(p.clone());
                        }
                        if let Some(p) = p_copy {
                            let _ = state_clone.tx.send(ServerMessage::PlayerUpdated { player: p });
                        }
                    }
                    ClientMessage::Move3D { x, y, z, ry, anim, model } => {
                        if let Some(p) = state_clone.players.write().await.get_mut(&pid) {
                            p.x = x;
                            p.y = y;
                            p.z = z;
                            p.ry = ry;
                            p.anim = anim.clone();
                            p.model = model.clone();
                        }
                        let _ = state_clone.tx.send(ServerMessage::PlayerMoved3D {
                            id: pid.clone(), x, y, z, ry, anim, model
                        });
                    }
                    ClientMessage::Interact => {
                        let p = state_clone.players.read().await.get(&pid).cloned();
                        if let Some(p) = p {
                            // Match interactions depending on player's current scene
                            let mut landmarks: Vec<(&str, f64, f64, &str)> = Vec::new();
                            
                            match p.scene.as_str() {
                                "overview" => {
                                    landmarks = vec![
                                        ("moses", 120.0, 310.0, "【摩西 · 旷野指引】\n法老追兵在后，红海在前。族人在旷野漫游中，如何保持道德与精神的统一？\n\n答案提示：坚守羊皮典籍与律法教育，将精神信念代代相传。（按快捷栏可进入西奈分场景探索）"),
                                        ("oasis", 270.0, 200.0, "【西奈绿洲 · 向导长老】\n在西奈旷野生存与迁徙中，最重要的生存智慧是什么？\n\n答案提示：珍惜有限清泉水源，老幼互助，严守族群契约与秩序。"),
                                        ("temple", 440.0, 110.0, "【所罗门圣殿 · 贤王所罗门】\n第一圣殿采用黎巴嫩雪松木精工建造，它象征着什么？\n\n答案提示：古代王国的黄金繁荣期，国家以信仰圣地凝聚人心，以智慧法度平息争端。（可进入圣殿内殿分场景）"),
                                        ("scholar", 690.0, 110.0, "【中世纪学者与旅商】\n流散欧洲被严令禁止拥有和耕种土地，犹太人开辟了什么生存生路？\n\n答案提示：“土地会被剥夺，但装在脑子里的知识谁也抢不走！” 崇尚读书教育、跨国商贸与记账金融。（可进入欧洲城镇分场景）"),
                                        ("wall", 430.0, 330.0, "【西墙守望老人】\n第二圣殿被罗马帝国摧毁后，哭墙承载着怎样的千年记忆？\n\n答案提示：在断壁残垣石缝中塞入对民族安宁的祈愿，抚石默念，不忘历史。"),
                                        ("chaplin", 700.0, 320.0, "【和平讲坛 · 查理·卓别林】\n在二战浩劫后的电影《大独裁者》中，卓别林向世人发出了怎样的呼吁？\n\n答案提示：用仁爱取代暴力，用理性破除偏见，唯有教育与同理心才是和平的归宿！"),
                                    ];
                                }
                                "sinai" => {
                                    landmarks = vec![
                                        ("sinai_moses", 200.0, 220.0, "【西奈圣山之巅 · 十诫石版】\n摩西登上雷鸣电闪的西奈山，带回了刻在两块石版上的十诫。这是人类最早强调什么精神的法典之一？\n\n答案提示：敬畏生命、孝敬父母、不可杀人偷盗、诚实作证——确立了古代平民伦理与公正律法的根基。"),
                                        ("sinai_tent", 620.0, 180.0, "【旷野营地讲堂】\n即使在漫无边际的流浪迁徙中，为何帐篷内始终点燃灯火教导孩童阅读？\n\n答案提示：文化传承优先于物资积累，让教育成为无国界、毁不掉的灵魂避难所。"),
                                        ("sinai_well", 420.0, 330.0, "【玛拉苦泉与甘泉】\n面对苦涩的水源与民众的抱怨，向导如何带领大家化危机为生机？\n\n答案提示：以信义聚合众心，通过科学寻水与合理配给度过缺水绝境。"),
                                    ];
                                }
                                "temple" => {
                                    landmarks = vec![
                                        ("temple_ark", 420.0, 130.0, "【至圣所 · 约柜金灯台】\n所罗门圣殿的至圣所中，存放的最神圣信物是什么？\n\n答案提示：盛放律法石版的约柜。它表明这个古国最崇高的不是君王本人的金雕玉砌，而是对律法与正义的永恒遵从。"),
                                        ("temple_court", 200.0, 300.0, "【所罗门法庭 · 两个母亲的争端】\n所罗门王以一把宝剑判决争婴案，展现了怎样的治国智慧？\n\n答案提示：洞察人性真情，利用母亲宁舍子不伤子的慈爱，让真相水落石出，以公正法度定纷止争。"),
                                        ("temple_cedar", 640.0, 300.0, "【黎巴嫩雪松木构】\n第一圣殿与推罗国王希兰合作采运雪松木，这反映了当时怎样的对外格局？\n\n答案提示：摒弃单一闭关自守，通过对外远洋航海与多边和平商贸，迎来地中海黄金贸易繁盛期。"),
                                    ];
                                }
                                "europe" => {
                                    landmarks = vec![
                                        ("europe_press", 220.0, 200.0, "【中世纪犹太印刷坊】\n当活字印刷术传入欧洲后，犹太学者与工匠为何废寝忘食地翻印多语言典籍？\n\n答案提示：将古典哲学、医学、几何学与希伯来文典籍译成拉丁文与阿拉伯文，成为东西方文明知识大交融的关键桥梁。"),
                                        ("europe_gate", 420.0, 120.0, "【隔都城门之限】\n中世纪欧洲部分领主下令在日落后锁闭犹太隔都（Ghetto）大门，面对严苛环境他们如何保持族群生机？\n\n答案提示：以自治公社互助相扶，设立无偿借贷社、孤儿院与救济金，以坚不可摧的社区凝聚力共渡严寒。"),
                                        ("europe_bank", 620.0, 280.0, "【长途汇票与商贸账册】\n在跨国旅行极易遭遇强盗劫掠金币的动荡年代，流散商人们首创了怎样的金融制度？\n\n答案提示：跨区域信贷信用汇票（Bill of Exchange），凭借横跨地中海的家族信用网络，开启了现代国际金融与结算的先声。"),
                                    ];
                                }
                                "peace" => {
                                    landmarks = vec![
                                        ("peace_chaplin", 220.0, 230.0, "【查理·卓别林《大独裁者》演说台】\n“士兵们！不要为暴政卖命，要为自由而战！”卓别林在演说中对人类未来寄托了什么厚望？\n\n答案提示：科技与知识本应使人富足自由，而不应带来冷酷残暴；唯有同理心、仁慈与宽容，才能让人类文明走出战争苦海。"),
                                        ("peace_olive", 620.0, 230.0, "【和平橄榄树纪念林】\n在战火与苦难历经沧桑之后，两族人民共同栽种下的这株橄榄树意味着什么？\n\n答案提示：放下刻骨世仇，铭记历史创伤而不延续仇恨，共同在迦南土地上耕耘两族共荣的安居家园。"),
                                    ];
                                }
                                _ => {}
                            }

                            for (id, ax, ay, q) in landmarks {
                                let dist = ((p.x - ax).powi(2) + (p.y - ay).powi(2)).sqrt();
                                if dist < 70.0 {
                                    let _ = state_clone.tx.send(ServerMessage::ShowSlideQuestion {
                                        target_id: pid.clone(),
                                        artifact_id: id.to_string(),
                                        question: q.to_string(),
                                    });
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    }

    state.players.write().await.remove(&player_id);
    let _ = state.tx.send(ServerMessage::PlayerLeft { id: player_id });
}

fn rand_seed(s: &str) -> usize {
    s.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize))
}
