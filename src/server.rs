use crate::data_model;
use crate::world;

use rocket_contrib::json::Json;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

pub type IpList = Vec<std::net::IpAddr>;
pub type SharedWorld = Arc<Mutex<world::World>>;

fn check_ban(user_ip: std::net::IpAddr, banned_ips: IpList) -> Result<(), data_model::WorldError> {
    if banned_ips.contains(&user_ip) {
        Err(data_model::WorldError::Other(String::from(
            "You have been struck by the holy Banhammer, now get out!",
        )))
    } else {
        Ok(())
    }
}

#[post("/connect")]
fn connect(
    world: rocket::State<SharedWorld>,
    banned_ips: rocket::State<IpList>,
    socket_addr: std::net::SocketAddr,
) -> Result<Json<data_model::Status>, data_model::WorldError> {
    check_ban(socket_addr.ip(), banned_ips.inner().clone())?;
    let mut world = world.lock().unwrap();
    Ok(Json(world.connect()?))
}

#[get("/<guid>/regarder")]
fn look_room(
    world: rocket::State<SharedWorld>,
    guid: String,
    banned_ips: rocket::State<IpList>,
    socket_addr: std::net::SocketAddr,
) -> Result<Json<data_model::Room>, data_model::WorldError> {
    check_ban(socket_addr.ip(), banned_ips.inner().clone())?;
    let mut world = world.lock().unwrap();
    match world.look(guid.clone()) {
        Ok(res) => Ok(Json(res)),
        Err(error) => Err(error.check_not_found(guid)),
    }
}

#[post("/<guid>/deplacement", data = "<req_direction>")]
fn movement(
    world: rocket::State<SharedWorld>,
    guid: String,
    req_direction: Json<data_model::ReqDirection>,
    banned_ips: rocket::State<IpList>,
    socket_addr: std::net::SocketAddr,
) -> Result<Json<data_model::Room>, data_model::WorldError> {
    check_ban(socket_addr.ip(), banned_ips.inner().clone())?;
    let mut world = world.lock().unwrap();
    match world.r#move(guid.clone(), req_direction.into_inner().direction) {
        Ok(res) => Ok(Json(res)),
        Err(error) => Err(error.check_not_found(guid)),
    }
}

#[get("/<guid>/examiner/<guid_dest>")]
fn look_entity(
    world: rocket::State<SharedWorld>,
    guid: String,
    guid_dest: String,
    banned_ips: rocket::State<IpList>,
    socket_addr: std::net::SocketAddr,
) -> Result<Json<data_model::Entity>, data_model::WorldError> {
    check_ban(socket_addr.ip(), banned_ips.inner().clone())?;
    let mut world = world.lock().unwrap();
    match world.look_entity(guid.clone(), guid_dest) {
        Ok(res) => Ok(Json(res)),
        Err(error) => Err(error.check_not_found(guid)),
    }
}

#[post("/<guid>/taper/<guid_dest>")]
fn attack(
    world: rocket::State<SharedWorld>,
    guid: String,
    guid_dest: String,
    banned_ips: rocket::State<IpList>,
    socket_addr: std::net::SocketAddr,
) -> Result<Json<data_model::Fight>, data_model::WorldError> {
    check_ban(socket_addr.ip(), banned_ips.inner().clone())?;
    let mut world = world.lock().unwrap();
    match world.attack(guid.clone(), guid_dest) {
        Ok(res) => Ok(Json(res)),
        Err(error) => Err(error.check_not_found(guid)),
    }
}

fn spawn_afk_thread(world: SharedWorld) {
    let check_rate = time::Duration::from_secs(5);

    thread::spawn(move || {
        loop {
            thread::sleep(check_rate);
            world.lock().unwrap().disconnect_afk_players().unwrap();
        }
    });
}

pub fn launch(world: world::World, banned_ips: IpList) -> Result<(), std::net::AddrParseError> {
    let world = Arc::new(Mutex::new(world));

    spawn_afk_thread(Arc::clone(&world));

    println!("{:?}", banned_ips);

    rocket::ignite()
        .manage(world)
        .manage(banned_ips)
        .mount(
            "/",
            routes![connect, look_room, movement, look_entity, attack],
        )
        .launch();

    Ok(())
}
