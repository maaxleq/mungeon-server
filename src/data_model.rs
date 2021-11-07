use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum WorldError {
    Dead,
    Wall,
    DiffRoom,
    Other(String),
}

impl WorldError {
    pub fn to_json_string(&self) -> String {
        match self {
            WorldError::Dead => String::from(r#"{ "type": "MORT", "message": "You are dead" }"#),
            WorldError::Wall => {
                String::from(r#"{ "type": "MUR", "message": "You bumped into a wall" }"#)
            }
            WorldError::DiffRoom => {
                String::from(r#"{ "type": "DIFFSALLE", "message": "Room mismatch" }"#)
            }
            WorldError::Other(msg) => format!("{{ \"message\": \"{}\" }}", msg),
        }
    }
}

impl<'r> rocket::response::Responder<'r> for WorldError {
    fn respond_to(self, req: &rocket::Request<'_>) -> rocket::response::Result<'r> {
        rocket::Response::build_from(self.to_json_string().respond_to(req).unwrap())
            .header(rocket::http::ContentType::JSON)
            .status(rocket::http::Status::raw(409))
            .ok()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Fighter {
    pub guid: String,
    #[serde(rename = "degats")]
    pub dp: u32,
    #[serde(rename = "vie")]
    pub hp: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Fight {
    #[serde(rename = "attaquant")]
    pub attacker: Fighter,
    #[serde(rename = "attaque")]
    pub defender: Fighter,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Room {
    pub description: String,
    #[serde(rename = "passages")]
    pub paths: Vec<Direction>,
    #[serde(rename = "entites")]
    pub entities: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Direction {
    #[serde(rename = "N")]
    N,
    #[serde(rename = "E")]
    E,
    #[serde(rename = "S")]
    S,
    #[serde(rename = "W")]
    W,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct ReqDirection {
    pub direction: Direction,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entity {
    pub description: String,
    pub r#type: EntityType,
    #[serde(rename = "vie")]
    pub life: u32,
    #[serde(rename = "totalvie")]
    pub total_life: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum EntityType {
    #[serde(rename = "MONSTRE")]
    Monster,
    #[serde(rename = "JOUEUR")]
    Player,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Status {
    pub guid: String,
    #[serde(rename = "totalvie")]
    pub total_life: u32,
    #[serde(rename = "salle")]
    pub room: Room,
}
