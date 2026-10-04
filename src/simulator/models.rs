use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlayerClass {
    Scholar,
    Explorer,
    Merchant,
}

impl Default for PlayerClass {
    fn default() -> Self {
        Self::Explorer
    }
}

impl PlayerClass {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Scholar => "学者",
            Self::Explorer => "探索者",
            Self::Merchant => "商人",
        }
    }

    pub fn perk(&self) -> &'static str {
        match self {
            Self::Scholar => "智慧试炼经验 +50%",
            Self::Explorer => "移动速度 +35% & 视野范围加倍",
            Self::Merchant => "开辟跨国商路 & 知识加成",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerState {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub ry: f64,
    pub anim: String,
    pub model: String,
    pub color: String,
    pub class: PlayerClass,
    pub level: u32,
    pub knowledge: u32,
    pub scene: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Move { dx: f64, dy: f64 },
    SetClass(PlayerClass),
    ChangeScene { scene: String, x: f64, y: f64 },
    LevelUp,
    Interact,
    Move3D { x: f64, y: f64, z: f64, ry: f64, anim: String, model: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Welcome { id: String },
    StateSync { players: Vec<PlayerState> },
    PlayerJoined { player: PlayerState },
    PlayerLeft { id: String },
    PlayerMoved { id: String, x: f64, y: f64 },
    PlayerUpdated { player: PlayerState },
    ShowSlideQuestion { target_id: String, artifact_id: String, question: String },
    PlayerMoved3D { id: String, x: f64, y: f64, z: f64, ry: f64, anim: String, model: String },
}
