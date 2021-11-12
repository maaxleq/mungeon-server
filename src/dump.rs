use crate::gen_plan;

use image;

pub fn dump_world(world_plan: &gen_plan::WorldPlan) {
    let width = world_plan.get_width();
    let height = world_plan.get_height();

    let x_offset = world_plan.get_x_offset();
    let y_offset = world_plan.get_y_offset();

    let mut img_buf = image::ImageBuffer::new(width as u32, height as u32);

    for (_, _, pixel) in img_buf.enumerate_pixels_mut() {
        *pixel = image::Rgb([0u8, 0u8, 0u8]);
    }

    for room in &world_plan.rooms {
        let x = (room.x + x_offset as isize) as u32;
        let y = (room.y + y_offset as isize) as u32;

        let pixel = img_buf.get_pixel_mut(x, y);
        if room.x == world_plan.spawn_x && room.y == world_plan.spawn_y {
            *pixel = image::Rgb([255u8, 0u8, 0u8]);
        } else {
            if x % 2 == y % 2 {
                *pixel = image::Rgb([50u8, 255u8, 50u8]);
            } else {
                *pixel = image::Rgb([50u8, 200u8, 50u8]);
            }
        }
    }

    img_buf.save("world_dump.png").unwrap();

    println!("Dumped map");
}
