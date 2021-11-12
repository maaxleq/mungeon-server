use crate::data_model;
use crate::gen_plan;

use std::collections::HashMap;

use std::time;

mod guid {
    pub fn random_guid() -> String {
        uuid::Uuid::new_v4().to_simple().to_string()
    }
}

#[derive(Clone, Debug)]
pub enum EntityType {
    Monster(String),
    Player,
}

#[derive(Clone, Debug)]
pub struct Entity {
    pub r#type: EntityType,
    pub max_hp: u32,
    pub hp: u32,
    pub dp: u32,
    pub location: Coords,
    pub afk_since: time::Instant,
}

impl Entity {
    fn random_monster(location: Coords) -> Entity {
        let hp = fastrand::u32(30..150);

        Entity {
            r#type: EntityType::Monster(String::from("A simple monster")),
            max_hp: hp,
            hp: hp,
            dp: fastrand::u32(10..30),
            location: location,
            afk_since: time::Instant::now(),
        }
    }

    fn generate_monster(
        location: Coords,
        defined_monster_plan: gen_plan::DefinedMonsterPlan,
    ) -> Entity {
        Entity {
            r#type: EntityType::Monster(
                defined_monster_plan
                    .description
                    .unwrap_or(String::from("A simple monster")),
            ),
            max_hp: defined_monster_plan.hp,
            hp: defined_monster_plan.hp,
            dp: defined_monster_plan.dp,
            location: location,
            afk_since: time::Instant::now(),
        }
    }

    fn generate_monsters(location: Coords, monsters_plan: gen_plan::MonstersPlan) -> Vec<Entity> {
        match monsters_plan {
            gen_plan::MonstersPlan::Random(length) => {
                let mut random_monsters: Vec<Entity> = Vec::new();

                for _ in 0..length {
                    random_monsters.push(Entity::random_monster(location.clone()));
                }

                random_monsters
            }
            gen_plan::MonstersPlan::Defined(monster_plans) => {
                let mut defined_monsters: Vec<Entity> = Vec::new();

                for monster_plan in monster_plans.iter() {
                    defined_monsters.push(Entity::generate_monster(
                        location.clone(),
                        monster_plan.clone(),
                    ));
                }

                defined_monsters
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Room {
    pub description: String,
    pub guids: Vec<String>,
    pub hp_regen: Option<u32>,
}

impl Room {
    pub fn generate_with_coords_and_entities(
        room_plan: gen_plan::RoomPlan,
    ) -> (Coords, Room, HashMap<String, Entity>) {
        let coords = Coords {
            x: room_plan.x,
            y: room_plan.y,
        };

        let description = room_plan.description.unwrap_or(String::from("A room"));
        let mut guids: Vec<String> = Vec::new();
        let mut entities: HashMap<String, Entity> = HashMap::new();

        match room_plan.monsters {
            Some(monsters_plan) => {
                let monsters = Entity::generate_monsters(coords.clone(), monsters_plan);

                for monster in monsters.iter() {
                    let guid = guid::random_guid();
                    guids.push(guid.clone());
                    entities.insert(guid, monster.clone());
                }
            }
            None => (),
        }

        (
            coords,
            Room {
                description: description,
                guids: guids,
                hp_regen: room_plan.hp_regen,
            },
            entities,
        )
    }

    pub fn remove_guid(&mut self, guid: String) -> Result<(), data_model::WorldError> {
        let mut found = false;

        for (i, g) in self.guids.clone().iter().enumerate() {
            if g == &guid {
                self.guids.remove(i);
                found = true;
            }
        }

        if found {
            Ok(())
        } else {
            Err(data_model::WorldError::DiffRoom)
        }
    }

    pub fn add_guid(&mut self, guid: String) -> Result<(), data_model::WorldError> {
        if self.guids.contains(&guid) {
            Err(data_model::WorldError::Other(String::from(
                "Entity is already present in this room!",
            )))
        } else {
            self.guids.push(guid);
            Ok(())
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Coords {
    x: isize,
    y: isize,
}

impl Coords {
    pub fn north(&self) -> Option<Coords> {
        Some(Coords {
            x: self.x,
            y: self.y.checked_add(1)?,
        })
    }

    pub fn south(&self) -> Option<Coords> {
        Some(Coords {
            x: self.x,
            y: self.y.checked_sub(1)?,
        })
    }

    pub fn east(&self) -> Option<Coords> {
        Some(Coords {
            x: self.x.checked_add(1)?,
            y: self.y,
        })
    }

    pub fn west(&self) -> Option<Coords> {
        Some(Coords {
            x: self.x.checked_sub(1)?,
            y: self.y,
        })
    }
}

#[derive(Clone, Debug)]
pub struct World {
    pub rooms: HashMap<Coords, Room>,
    pub entities: HashMap<String, Entity>,
    pub spawn: Coords,
    pub afk_threshold: time::Duration,
}

impl World {
    pub fn new() -> World {
        World {
            rooms: HashMap::new(),
            entities: HashMap::new(),
            spawn: Coords { x: 0, y: 0 },
            afk_threshold: time::Duration::from_secs(60),
        }
    }

    pub fn generate(world_plan: &gen_plan::WorldPlan) -> World {
        let mut world = World::new();

        for room_plan in world_plan.rooms.iter() {
            let (coords, room, entities) =
                Room::generate_with_coords_and_entities(room_plan.clone());

            world.rooms.insert(coords, room);
            world.entities.extend(entities);
        }

        world.spawn.x = world_plan.spawn_x;
        world.spawn.y = world_plan.spawn_y;

        world
    }

    pub fn get_entity(&mut self, guid: String) -> Result<&mut Entity, data_model::WorldError> {
        match self.entities.get_mut(&guid) {
            Some(entity) => Ok(entity),
            None => Err(data_model::WorldError::EntityNotFound(guid)),
        }
    }

    pub fn get_room(&mut self, location: Coords) -> Result<&mut Room, data_model::WorldError> {
        match self.rooms.get_mut(&location) {
            Some(room) => Ok(room),
            None => Err(data_model::WorldError::Wall),
        }
    }

    pub fn disconnect_afk_players(&mut self) -> Result<(), data_model::WorldError> {
        let mut entities_to_remove: Vec<(Coords, String)> = Vec::new();

        for entity in self.entities.iter_mut() {
            match entity.1.r#type {
                EntityType::Monster(_) => (),
                EntityType::Player => {
                    let guid = entity.0.clone();
                    let location = entity.1.location.clone();
                    if time::Instant::now() - entity.1.afk_since > self.afk_threshold {
                        entities_to_remove.push((location, guid));
                    }
                }
            }
        }

        for (location, guid) in entities_to_remove.iter() {
            self.get_room(location.clone())?.remove_guid(guid.clone())?;
            self.entities.remove(guid);
        }

        Ok(())
    }

    pub fn connect(&mut self) -> Result<data_model::Status, data_model::WorldError> {
        let coords = self.spawn.clone();
        let room = self.get_room(coords.clone())?;
        let guid = guid::random_guid();
        let max_hp = 100;

        let player = Entity {
            dp: 10,
            hp: max_hp,
            max_hp: max_hp,
            location: coords.clone(),
            r#type: EntityType::Player,
            afk_since: time::Instant::now(),
        };

        room.add_guid(guid.clone())?;

        let description = room.description.clone();
        let guids = room.guids.clone();

        self.entities.insert(guid.clone(), player);

        Ok(data_model::Status {
            guid: guid.clone(),
            total_life: max_hp,
            room: data_model::Room {
                description: description,
                entities: guids,
                paths: self.get_directions_for_coordinates(coords.clone()),
            },
        })
    }

    fn player_acted(&mut self, guid: String) -> Result<(), data_model::WorldError> {
        self.get_entity(guid)?.afk_since = time::Instant::now();

        Ok(())
    }

    pub fn look(&mut self, guid: String) -> Result<data_model::Room, data_model::WorldError> {
        let coords = self.get_entity(guid)?.location.clone();
        let room = self.get_room(coords.clone())?;

        Ok(data_model::Room {
            description: room.description.clone(),
            entities: room.guids.clone(),
            paths: self.get_directions_for_coordinates(coords),
        })
    }

    pub fn look_entity(
        &mut self,
        guid: String,
        guid_dest: String,
    ) -> Result<data_model::Entity, data_model::WorldError> {
        let coords = self.get_entity(guid)?.location.clone();
        let entity = self.get_entity(guid_dest.clone())?;

        if coords != entity.location {
            return Err(data_model::WorldError::DiffRoom);
        } else {
            Ok(data_model::Entity {
                description: match entity.r#type.clone() {
                    EntityType::Player => String::from("Another player"),
                    EntityType::Monster(description) => description,
                },
                life: entity.hp,
                total_life: entity.max_hp,
                r#type: match entity.r#type {
                    EntityType::Player => data_model::EntityType::Player,
                    EntityType::Monster(_) => data_model::EntityType::Monster,
                },
            })
        }
    }

    pub fn r#move(
        &mut self,
        guid: String,
        direction: data_model::Direction,
    ) -> Result<data_model::Room, data_model::WorldError> {
        self.player_acted(guid.clone())?;

        let coords = self.get_entity(guid.clone())?.location.clone();
        let mut new_coords = coords.clone();
        match direction {
            data_model::Direction::N => new_coords.y += 1,
            data_model::Direction::S => new_coords.y -= 1,
            data_model::Direction::E => new_coords.x += 1,
            data_model::Direction::W => new_coords.x -= 1,
        }
        self.get_room(new_coords.clone())?;
        let prev_room = self.get_room(coords.clone())?;
        prev_room.remove_guid(guid.clone())?;
        let next_room = self.get_room(new_coords.clone())?;
        next_room.add_guid(guid.clone())?;
        let guids = next_room.guids.clone();
        let cloned_guids = guids.clone();
        let description = next_room.description.clone();
        let hp_regen = next_room.hp_regen.clone();
        let hp = self.get_entity(guid.clone())?.hp.clone();
        let max_hp = self.get_entity(guid.clone())?.max_hp.clone();
        let new_hp = match hp_regen {
            Some(value) => std::cmp::min(hp + value, max_hp),
            None => hp.clone(),
        };
        self.get_entity(guid.clone())?.hp = new_hp;
        self.get_entity(guid.clone())?.location = new_coords.clone();

        Ok(data_model::Room {
            description: description,
            entities: cloned_guids,
            paths: self.get_directions_for_coordinates(new_coords),
        })
    }

    fn get_directions_for_coordinates(&mut self, coords: Coords) -> Vec<data_model::Direction> {
        let mut directions = Vec::new();

        match coords.north() {
            Some(new_coords) => match self.get_room(new_coords) {
                Ok(_) => directions.push(data_model::Direction::N),
                Err(_) => (),
            },
            None => (),
        }

        match coords.south() {
            Some(new_coords) => match self.get_room(new_coords) {
                Ok(_) => directions.push(data_model::Direction::S),
                Err(_) => (),
            },
            None => (),
        }

        match coords.east() {
            Some(new_coords) => match self.get_room(new_coords) {
                Ok(_) => directions.push(data_model::Direction::E),
                Err(_) => (),
            },
            None => (),
        }

        match coords.west() {
            Some(new_coords) => match self.get_room(new_coords) {
                Ok(_) => directions.push(data_model::Direction::W),
                Err(_) => (),
            },
            None => (),
        }

        directions
    }

    pub fn attack(
        &mut self,
        attacker_guid: String,
        defender_guid: String,
    ) -> Result<data_model::Fight, data_model::WorldError> {
        self.player_acted(attacker_guid.clone())?;

        let mut attacker = self.get_entity(attacker_guid.clone())?.clone();
        let mut defender = self.get_entity(defender_guid.clone())?.clone();

        if attacker.location != defender.location {
            Err(data_model::WorldError::DiffRoom)
        } else {
            attacker.hp = attacker.hp.saturating_sub(defender.dp);
            defender.hp = defender.hp.saturating_sub(attacker.dp);

            let dead = attacker.hp == 0;

            self.get_entity(attacker_guid.clone())?.hp = attacker.hp;

            if dead {
                self.get_room(attacker.location)?
                    .remove_guid(attacker_guid.clone())?;
                self.entities.remove(&attacker_guid);
            }

            self.get_entity(defender_guid.clone())?.hp = defender.hp;

            if defender.hp == 0 {
                self.get_room(defender.location)?
                    .remove_guid(defender_guid.clone())?;
                self.entities.remove(&defender_guid);
            } else {
                self.get_entity(defender_guid.clone())?.hp = defender.hp;
            }

            if dead {
                Err(data_model::WorldError::Disappeared)
            } else {
                Ok(data_model::Fight {
                    attacker: data_model::Fighter {
                        guid: attacker_guid,
                        dp: attacker.dp,
                        hp: attacker.hp,
                    },
                    defender: data_model::Fighter {
                        guid: defender_guid,
                        dp: defender.dp,
                        hp: defender.hp,
                    },
                })
            }
        }
    }
}
