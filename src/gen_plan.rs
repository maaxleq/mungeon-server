use serde::{Deserialize, Serialize};

use crate::seed;

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
    pub x: isize,
    pub y: isize,
    pub description: Option<String>,
    pub monsters: Option<MonstersPlan>,
    pub hp_regen: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldPlan {
    pub rooms: Vec<RoomPlan>,
    pub spawn_x: isize,
    pub spawn_y: isize,
}

impl WorldPlan {
    pub fn new() -> WorldPlan {
        WorldPlan {
            rooms: Vec::new(),
            spawn_x: 0,
            spawn_y: 0,
        }
    }

    pub fn get_width(&self) -> usize {
        let mut min_x: isize = 0;
        let mut max_x: isize = 0;

        for room in &self.rooms {
            if room.x < min_x {
                min_x = room.x;
            }
            if room.x > max_x {
                max_x = room.x;
            }
        }

        (max_x - min_x + 1) as usize
    }

    pub fn get_height(&self) -> usize {
        let mut min_y: isize = 0;
        let mut max_y: isize = 0;

        for room in &self.rooms {
            if room.y < min_y {
                min_y = room.y;
            }
            if room.y > max_y {
                max_y = room.y;
            }
        }

        (max_y - min_y + 1) as usize
    }

    pub fn get_x_offset(&self) -> usize {
        let mut min_x: isize = 0;

        for room in &self.rooms {
            if room.x < min_x {
                min_x = room.x;
            }
        }

        (-min_x) as usize
    }

    pub fn get_y_offset(&self) -> usize {
        let mut min_y: isize = 0;

        for room in &self.rooms {
            if room.y < min_y {
                min_y = room.y;
            }
        }

        (-min_y) as usize
    }

    pub fn from_seeder(mut seeder: seed::Seeder) -> WorldPlan {
        println!("Generating map");

        let mut world_plan = WorldPlan::new();

        let corridor_count = seeder.seed_u32_bounded(150, 600);
        let mut coords_list: Vec<(isize, isize)> = vec![(0, 0)];

        let mut direction = 0;

        world_plan.rooms.push(RoomPlan {
            x: 0,
            y: 0,
            description: None,
            monsters: None,
            hp_regen: None,
        });

        println!("Generated spawn");

        for i in 0..corridor_count {
            let index = seeder.seed_u32_bounded(0, (coords_list.len() - 1) as u32) as usize;
            let corridor_length = seeder.seed_u32_bounded(50, 150) as usize;
            direction = (direction + 1) % 4;
            let vector_function = WorldPlan::get_vector_function(direction);
            let mut coords = coords_list[index];

            for _ in 0..corridor_length {
                coords = vector_function(coords);
                coords_list.push(coords.clone());
                if coords.0 != 0 || coords.1 != 0 {
                    let difficulty_multiplier = WorldPlan::get_difficulty_multiplier(
                        corridor_count.clone() * 30,
                        coords.clone(),
                    );
                    let hp_regen =
                        Some(seeder.seed_u32_bounded(0, 1) as u32 * difficulty_multiplier);

                    world_plan.rooms.push(RoomPlan {
                        x: coords.0,
                        y: coords.1,
                        description: Some(format!(
                            "You are at coordinates ({},{}). This room has a difficulty of {}{}",
                            coords.0,
                            coords.1,
                            difficulty_multiplier,
                            match hp_regen {
                                None => String::from(""),
                                Some(0) => String::from(""),
                                Some(value) => format!(" and regenerates {} HP", value),
                            }
                        )),
                        monsters: Some(MonstersPlan::Random(
                            seeder.seed_u32_bounded(1, 3) as usize * difficulty_multiplier as usize,
                        )),
                        hp_regen: hp_regen,
                    });
                }
            }

            println!("Generated corridor {}/{}", i + 1, corridor_count);
        }

        let room_count = world_plan.rooms.len();

        world_plan.rooms[0].description = Some(format!(
            "Welcome! This is the spawn room. There are {} other {} to explore",
            room_count - 1,
            if room_count - 1 > 1 { "rooms" } else { "room" },
        ));

        println!("Finished generating map");

        world_plan
    }

    fn get_vector_function(direction: u32) -> fn((isize, isize)) -> (isize, isize) {
        match direction % 4 {
            0 => (|coords: (isize, isize)| (coords.0, coords.1 + 1)),
            1 => (|coords: (isize, isize)| (coords.0, coords.1 - 1)),
            2 => (|coords: (isize, isize)| (coords.0 - 1, coords.1)),
            _ => (|coords: (isize, isize)| (coords.0 + 1, coords.1)),
        }
    }

    fn get_difficulty_multiplier(room_count: u32, coords: (isize, isize)) -> u32 {
        let distance_from_spawn = ((coords.0.pow(2) + coords.1.pow(2)) as f32).sqrt();
        let world_radius = (room_count as f32 / std::f32::consts::PI).sqrt();

        (distance_from_spawn / world_radius * 8f32).ceil() as u32
    }
}
