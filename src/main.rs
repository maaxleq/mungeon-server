#![feature(decl_macro)]

#[macro_use]
extern crate rocket;

mod data_model;
mod dump;
mod gen_plan;
mod seed;
mod server;
mod world;

use std::env;
use std::error;
use std::fs;

static ERROR_ARGUMENT_PARSE: &str = "Could not parse argument";
static NO_GEN_ERROR: &str = "No input file or seed provided";
static FILE_READ_ERROR: &str = "Could not read file";
static GEN_PARSE_ERROR: &str = "Error while parsing generation plan";

fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut filename = String::new();
    let mut data = String::new();
    let mut seed = String::new();
    let mut dump = false;

    for arg in args.iter().skip(1) {
        if arg.starts_with("gen=") {
            filename = arg[4..arg.len()].to_string();
            data =
                fs::read_to_string(&filename).expect(&format!("{} {}", FILE_READ_ERROR, &filename));
        } else if arg.starts_with("seed=") {
            let value = arg[5..arg.len()].to_string();
            if value == "random" {
                for _ in 0..50 {
                    seed.push(fastrand::digit(16));
                }
            } else {
                seed = value;
            }
        } else if arg == "--dump" {
            dump = true;
        } else {
            panic!("{} {}", ERROR_ARGUMENT_PARSE, arg);
        }
    }

    let world: world::World;
    let plan: gen_plan::WorldPlan;

    if filename == "" {
        if seed == "" {
            panic!("{}", NO_GEN_ERROR);
        } else {
            let seeder: seed::Seeder = seed::Seeder::try_from_seed(seed)?;
            plan = gen_plan::WorldPlan::from_seeder(seeder);
            world = world::World::generate(&plan);
        }
    } else {
        plan = serde_json::from_str(data.as_str()).expect(GEN_PARSE_ERROR);
        world = world::World::generate(&plan);
    }

    if dump {
        dump::dump_world(&plan);
    } else {
        server::launch(world);
    }

    Ok(())
}
