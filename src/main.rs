#![feature(decl_macro)]

#[macro_use]
extern crate rocket;

mod data_model;
mod gen_plan;
mod server;
mod world;

use std::env;
use std::error;
use std::fs;

static ERROR_ARGUMENT_PARSE: &str = "Could not parse argument";
static NO_INPUT_FILE_ERROR: &str = "No input file provided";
static FILE_READ_ERROR: &str = "Could not read file";
static GEN_PARSE_ERROR: &str = "Error while parsing generation plan";

fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut filename = String::new();
    let mut data = String::new();

    for arg in args.iter().skip(1) {
        if arg.starts_with("gen=") {
            filename = arg[4..arg.len()].to_string();
            data =
                fs::read_to_string(&filename).expect(&format!("{} {}", FILE_READ_ERROR, &filename));
        } else {
            panic!("{} {}", ERROR_ARGUMENT_PARSE, arg);
        }
    }

    if filename == "" {
        panic!("{}", NO_INPUT_FILE_ERROR);
    }

    let plan: gen_plan::WorldPlan = serde_json::from_str(data.as_str()).expect(GEN_PARSE_ERROR);
    let world = world::World::generate(plan);

    server::launch(world);

    Ok(())
}
