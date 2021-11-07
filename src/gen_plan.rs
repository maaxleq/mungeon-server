use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DefinedMonsterPlan {
    pub description: Option<String>,
    pub dp: u32,
    pub hp: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MonstersPlan {
    Defined(Vec<DefinedMonsterPlan>),
    Random(usize),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomPlan {
    pub x: usize,
    pub y: usize,
    pub description: Option<String>,
    pub monsters: Option<MonstersPlan>,
    pub hp_regen: Option<u32>,
    pub new_max_hp: Option<u32>,
    pub new_dp: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldPlan {
    pub rooms: Vec<RoomPlan>,
    pub spawn_x: usize,
    pub spawn_y: usize
}
