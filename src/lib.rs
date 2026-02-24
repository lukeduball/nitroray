use image::{Rgb, RgbImage};
use xenofrost::core::math::Vec3;

use crate::{camera::Camera, geometry::{Intersectable, Sphere}, ray::Ray};

mod camera;
mod geometry;
mod math;
mod ray;

pub fn run() {
    const WIDTH: usize = 480;
    const HEIGHT: usize = 270;
    let mut framebuffer = vec![Vec3::new(0.0, 0.0, 0.0); WIDTH*HEIGHT];

    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
    let camera = Camera::new(Vec3::splat(0.0), 0.0, 0.0, 90.0, aspect_ratio);
    let sphere = Sphere::new(Vec3::new(0.0, 0.0, 8.0), 1.0);


    let field_of_view_component = f32::tan(camera.get_field_of_view() / 2.0);
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            //pixel screen gets the center of each pixel and divides to put it in normalized coordinates between 0 and 1
            let pixel_screen_x = (x as f32 + 0.5) / WIDTH as f32;
            let pixel_screen_y = (y as f32 + 0.5) / HEIGHT as f32;
            let pixel_camera_x = (2.0 * pixel_screen_x - 1.0) * aspect_ratio * field_of_view_component;
            let pixel_camera_y = (1.0 - 2.0 * pixel_screen_y) * field_of_view_component;
            let pixel_coordinate = Vec3::new(pixel_camera_x, pixel_camera_y, 1.0);
            let world_coordinate = camera.convert_view_space_to_world_space(pixel_coordinate);
            let ray_direction = (world_coordinate - camera.get_origin()).normalize();
            let ray = Ray::new(world_coordinate, ray_direction);

            if sphere.intersect(ray) {
                framebuffer[x+y*WIDTH] = Vec3::new(1.0, 0.0, 0.0);
            }
        }
    }


    let mut out_image = RgbImage::new(WIDTH as u32, HEIGHT as u32);
    for x in 0..WIDTH {
        for y in 0..HEIGHT {
            let index = x + y*WIDTH;
            let pixel = framebuffer[index] * 255.0;
            let red = pixel.x as u8;
            let green = pixel.y as u8;
            let blue = pixel.z as u8;
            out_image.put_pixel(x as u32, y as u32, Rgb([red, green, blue]));
        } 
    }

    let _ = out_image.save("res/out.png");
}