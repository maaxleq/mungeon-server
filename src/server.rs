use crate::data_model;
use crate::world;

use rocket_contrib::json::Json;

use std::sync::Mutex;

pub type SharedWorld = Mutex<world::World>;

#[post("/connect")]
fn connect(world: rocket::State<SharedWorld>) -> Result<Json<data_model::Status>, data_model::WorldError> {
    let mut world = world.lock().unwrap();
    Ok(Json(world.connect()?))
}

#[get("/<guid>/regarder")]
fn look_room(world: rocket::State<SharedWorld>, guid: String) -> Result<Json<data_model::Room>, data_model::WorldError> {
    let mut world = world.lock().unwrap();
    Ok(Json(world.look(guid)?))
}

#[post("/<guid>/deplacement", data = "<req_direction>")]
fn movement(world: rocket::State<SharedWorld>, guid: String, req_direction: Json<data_model::ReqDirection>) -> Result<Json<data_model::Room>, data_model::WorldError> {
    let mut world = world.lock().unwrap();
    Ok(Json(world.r#move(guid, req_direction.into_inner().direction)?))
}

#[get("/<guid>/examiner/<guid_dest>")]
fn look_entity(world: rocket::State<SharedWorld>, guid: String, guid_dest: String) -> Result<Json<data_model::Entity>, data_model::WorldError> {
    let mut world = world.lock().unwrap();
    Ok(Json(world.look_entity(guid, guid_dest)?))
}

#[post("/<guid>/taper/<guid_dest>")]
fn attack(world: rocket::State<SharedWorld>, guid: String, guid_dest: String) -> Result<Json<data_model::Fight>, data_model::WorldError> {
    let mut world = world.lock().unwrap();
    Ok(Json(world.attack(guid, guid_dest)?))
}

pub fn launch(world: world::World) {
    rocket::ignite()
        .manage(Mutex::new(world))
        .mount("/", routes![connect, look_room, movement, look_entity, attack])
        .launch();
}
