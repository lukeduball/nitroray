use std::rc::Rc;

use image::{Rgb, RgbImage};
use xenofrost::core::math::Vec3;

use crate::{camera::Camera, geometry::Sphere, light::{DirectionalLight, Light}, material::Material, math::Transform3d, model::Model, object::{FaceIndex, Intersectable, ModelObject}, ray::Ray};

mod camera;
mod geometry;
mod light;
mod material;
mod math;
mod mesh;
mod model;
mod object;
mod ray;

fn get_color_from_raycast(ray: &Ray, object_list: &Vec<Box<dyn Intersectable>>, light_list: &Vec<Box<dyn Light>>) -> Vec3 {
    let background_color = Vec3::new(0.35, 0.35, 0.35);

    let (collision_object, distance_parameter, mesh_info) = find_ray_intersection_with_scene(ray, object_list);
    if let Some(object) = collision_object {
        let intersection_point = ray.get_origin() + ray.get_direction() * distance_parameter;

        let material_type = object.get_material_at_intersection(&intersection_point, &mesh_info).get_material_type();

        match material_type {
            material::MaterialType::Phong { diffuse_component, specular_component, power_component } => {
                let normal = object.get_normal_at_intersection(&intersection_point, &mesh_info);
                let color_at_intersection = object.get_color_at_intersection(&intersection_point, &mesh_info);

                for light in light_list {

                    let (light_direction, attenuated_light, _light_distance_parameter) = light.get_light_direction_intensity_and_distance_parameter(intersection_point);

                    let shadow_ray = Ray::new(intersection_point - light_direction * math::NITRORAY_FLOAT_EPSILON, -light_direction);
                    let (shadow_collision_object, _shadow_parameter, _mesh_info) = find_ray_intersection_with_scene(&shadow_ray, object_list);
                    if shadow_collision_object.is_none() {
                        let diffuse_color = color_at_intersection * attenuated_light * f32::max(0.0, normal.dot(-light_direction));
                        return diffuse_color;
                    }
                    else {
                        return Vec3::new(0.0, 0.0, 0.0);
                    }
                }
            },
            material::MaterialType::Reflect => todo!(),
            material::MaterialType::ReflectRefract { refraction_component } => todo!(),
        };
    }

    background_color
}

fn find_ray_intersection_with_scene<'a>(ray: &'a Ray, object_list: &'a Vec<Box<dyn Intersectable>>) -> (Option<&'a Box<dyn Intersectable>>, f32, Option<FaceIndex>) {
    let mut min_distance_parameter = f32::INFINITY;
    let mut collision_object: Option<&Box<dyn Intersectable>> = None;
    let mut mesh_info = None;
    for object in object_list {
        let result = object.intersect(ray);
        if result.does_intersect && result.intersection_parameter < min_distance_parameter {
            min_distance_parameter = result.intersection_parameter;
            collision_object = Some(object);
            mesh_info = result.mesh_info;
        }
    }
    
    (collision_object, min_distance_parameter, mesh_info)
}

pub fn run() {
    const WIDTH: usize = 480;
    const HEIGHT: usize = 270;
    let mut framebuffer = vec![Vec3::new(0.0, 0.0, 0.0); WIDTH*HEIGHT];

    let aspect_ratio = WIDTH as f32 / HEIGHT as f32;
    let camera = Camera::new(Vec3::new(0.0, 0.0, 0.0), 0.0, 0.0, 90.0, aspect_ratio);

    let plane_model = Rc::new(Model::load_model("res/models/plane.gltf"));

    let mut light_list: Vec<Box<dyn Light>> = Vec::new();
    light_list.push(Box::new(DirectionalLight::new(Vec3::new(-1.0, -1.0, 1.0).normalize(), Vec3::new(1.0, 1.0, 1.0), 1.0)));

    let red_material = Material::new(Vec3::new(1.0, 0.0, 0.0), material::MaterialType::Phong { diffuse_component: 1.0, specular_component: 0.0, power_component: 0.0 });
    let green_material = Material::new(Vec3::new(0.0, 1.0, 0.0), material::MaterialType::Phong { diffuse_component: 1.0, specular_component: 0.0, power_component: 0.0 });
    let blue_material = Material::new(Vec3::new(0.0, 0.0, 1.0), material::MaterialType::Phong { diffuse_component: 1.0, specular_component: 0.0, power_component: 0.0 });
    let purple_material = Material::new(Vec3::new(1.0, 0.0, 1.0), material::MaterialType::Phong { diffuse_component: 1.0, specular_component: 0.0, power_component: 0.0 });
    let yellow_material = Material::new(Vec3::new(1.0, 1.0, 0.0), material::MaterialType::Phong { diffuse_component: 1.0, specular_component: 0.0, power_component: 0.0 });

    let mut object_list: Vec<Box<dyn Intersectable>> = Vec::new();
    object_list.push(Box::new(Sphere::new(Vec3::new(0.0, 0.0, 5.0), 1.0, red_material)));
    object_list.push(Box::new(Sphere::new(Vec3::new(1.5, 0.0, 4.0), 1.0, green_material)));
    object_list.push(Box::new(Sphere::new(Vec3::new(0.0, 3.0, 6.0), 1.0, blue_material)));
    object_list.push(Box::new(Sphere::new(Vec3::new(-4.0, 0.0, 5.0), 1.0, purple_material)));
    object_list.push(Box::new(ModelObject::new(
        Transform3d::new(Vec3::new(0.0, -1.0, 4.0), 0.0, 0.0, 0.0, Vec3::splat(2.0)), 
        yellow_material, 
        plane_model.clone())
    ));

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

            framebuffer[x+y*WIDTH] = get_color_from_raycast(&ray, &object_list, &light_list);
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