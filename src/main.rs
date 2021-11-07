#![feature(decl_macro)]

#[macro_use] extern crate rocket;

mod gen_plan;
mod world;
mod data_model;
mod server;

fn main() {
    let data = r#"
        {
            "spawn_x": 0,
            "spawn_y": 0,
            "rooms": [
                {
                    "x": 0,
                    "y": 0
                },
                {
                    "x": 1,
                    "y": 0,
                    "description": "a room",
                    "monsters": 5
                },
                {
                    "x": 2,
                    "y": 0,
                    "description": "another room",
                    "monsters": [
                        {
                            "descrition": "a monster",
                            "hp": 50,
                            "dp": 5
                        },
                        {
                            "descrition": "another monster",
                            "hp": 50,
                            "dp": 5
                        },
                        {
                            "hp": 50,
                            "dp": 5
                        }
                    ]
                }
            ]
        }
    "#;

    let plan: gen_plan::WorldPlan = serde_json::from_str(data).unwrap();
    let world = world::World::generate(plan);

    server::launch(world);
}
