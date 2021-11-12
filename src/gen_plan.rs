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

        let room_count = seeder.seed_u32_bounded(150, 1000);
        let mut coords_list: Vec<(isize, isize)> = vec![(0, 0)];

        world_plan.rooms.push(RoomPlan {
            x: 0,
            y: 0,
            description: Some(format!(
                "Welcome! This is the spawn room. There are {} other {} to explore",
                room_count - 1,
                if room_count - 1 > 1 { "rooms" } else { "room" },
            )),
            monsters: None,
            hp_regen: None,
        });

        println!("Generated room 1/{} (spawn)", room_count);

        for i in 1..room_count {
            let coords = WorldPlan::get_coords_for_new_room(&coords_list, &mut seeder);
            coords_list.push(coords.clone());
            let difficulty_multiplier =
                WorldPlan::get_difficulty_multiplier(room_count.clone(), coords.clone());
            let hp_regen = Some(seeder.seed_u32_bounded(0, 1) as u32 * difficulty_multiplier);

            world_plan.rooms.push(RoomPlan {
                x: coords.0,
                y: coords.1,
                description: Some(format!(
                    "You are at coordinates ({},{}). This room has a difficulty of {}{}",
                    coords.0, coords.1, difficulty_multiplier, match hp_regen {
                        None => String::from(""),
                        Some(0) => String::from(""),
                        Some(value) => format!(" and regenerates {} HP", value)
                    }
                )),
                monsters: Some(MonstersPlan::Random(
                    seeder.seed_u32_bounded(1, 3) as usize * difficulty_multiplier as usize,
                )),
                hp_regen: hp_regen,
            });

            println!("Generated room {}/{}", i + 1, room_count);
        }

        println!("Finished generating map");

        world_plan
    }

    fn get_difficulty_multiplier(room_count: u32, coords: (isize, isize)) -> u32 {
        let distance_from_spawn = ((coords.0.pow(2) + coords.1.pow(2)) as f32).sqrt();
        let world_radius = (room_count as f32 / std::f32::consts::PI).sqrt();

        (distance_from_spawn / world_radius * 8f32).ceil() as u32
    }

    fn get_coords_from_seed(coords: (isize, isize), seed: u32) -> (isize, isize) {
        match seed % 4 {
            0 => (coords.0, coords.1 + 1),
            1 => (coords.0, coords.1 - 1),
            2 => (coords.0 - 1, coords.1),
            _ => (coords.0 + 1, coords.1),
        }
    }

    fn get_coords_for_new_room(
        coords_list: &Vec<(isize, isize)>,
        seeder: &mut seed::Seeder,
    ) -> (isize, isize) {
        let mut found = false;
        let mut res_coords = (0, 0);
        let mut index = seeder.seed_u32_bounded(0, (coords_list.len() - 1) as u32) as usize;

        while !found {
            let coords = coords_list[index];
            let direction = seeder.seed_u32_bounded(1, 4);
            let mut iteration = 1;
            let mut new_coords = WorldPlan::get_coords_from_seed(coords, direction);
            while coords_list.contains(&new_coords) && iteration < 4 {
                new_coords = WorldPlan::get_coords_from_seed(coords, direction + iteration);
                iteration += 1;
            }
            found = !coords_list.contains(&new_coords);

            if found {
                res_coords = new_coords;
            } else {
                index = (index + 1) % coords_list.len();
            }
        }

        res_coords
    }
}
