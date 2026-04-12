use std::rc::Rc;

use image::{Rgb, RgbImage};
use xenofrost::core::math::Vec3;

use crate::{camera::Camera, geometry::Sphere, light::{DirectionalLight, Light}, material::{Material, MaterialType}, math::Transform3d, model::Model, object::{FaceIndex, Intersectable, ModelObject}, ray::Ray};

mod camera;
mod geometry;
mod light;
mod material;
mod math;
mod mesh;
mod model;
mod object;
mod ray;

const MAX_RAY_DEPTH: u32 = 3;
const BACKGROUND_COLOR: Vec3 = Vec3::new(0.35, 0.35, 0.35);
const REFLECTION_DIM_FACTOR: f32 = 0.8;

fn compute_fresnel(incident_direction: &Vec3, normal: &Vec3, refraction_component: f32) -> f32 {
    let incident_cos = incident_direction.dot(*normal).clamp(-1.0, 1.0);
    //the current medium refraction component, air has a value of 1
    let mut current_medium_refraction_component = 1.0;
    //the next medium of refraction component
    let mut next_medium_refraction_component = refraction_component;

    // The ray is starting inside the surface so the medium being traveled to and from need to be swapped
    if incident_cos < 0.0 {
        std::mem::swap(&mut current_medium_refraction_component, &mut next_medium_refraction_component);
    }

    let sin_angle_of_refraction = (current_medium_refraction_component / next_medium_refraction_component) * (1.0 - incident_cos*incident_cos).max(0.0).sqrt().min(1.0);
    let cos_angle_of_refraction = (1.0 - sin_angle_of_refraction*sin_angle_of_refraction).max(0.0).sqrt();
    let positive_incident_cos = incident_cos.abs();

    let parallel_fresnel = ((next_medium_refraction_component*positive_incident_cos) - (current_medium_refraction_component*cos_angle_of_refraction)) / ((next_medium_refraction_component*positive_incident_cos) + (current_medium_refraction_component*cos_angle_of_refraction));
    let perpendicular_fresnel = ((next_medium_refraction_component*cos_angle_of_refraction) - (current_medium_refraction_component*positive_incident_cos)) / ((next_medium_refraction_component*cos_angle_of_refraction) + (current_medium_refraction_component*positive_incident_cos));

    (parallel_fresnel * parallel_fresnel + perpendicular_fresnel * perpendicular_fresnel) * 0.5
}

fn get_refraction_vector(incident_direction: &Vec3, normal: &Vec3, refraction_component: f32) -> Vec3 {
    let mut incident_normal_cos = incident_direction.dot(*normal).clamp(-1.0, 1.0);
    
    //the current medium refraction component, air has a value of 1
    let mut current_medium_refraction_component = 1.0;
    //the next medium of refraction component
    let mut next_medium_refraction_component = refraction_component;

    let mut refraction_normal = *normal;
    // The ray is starting inside the surface so the medium being traveled to and from need to be swapped
    if incident_normal_cos < 0.0 {
        incident_normal_cos = -incident_normal_cos;
    }
    else {
        refraction_normal = -refraction_normal;
        std::mem::swap(&mut current_medium_refraction_component, &mut next_medium_refraction_component);
    }

    let refraction_component_ratio = current_medium_refraction_component / next_medium_refraction_component;
    let criticalValue = 1.0 - refraction_component_ratio * refraction_component_ratio * (1.0 - incident_normal_cos * incident_normal_cos);

    (refraction_component_ratio * incident_direction + (refraction_component_ratio * incident_normal_cos - criticalValue.sqrt()) * refraction_normal).normalize()
}

fn get_reflection_vector(incident_direction: &Vec3, normal: &Vec3) -> Vec3 {
    (incident_direction - 2.0 * incident_direction.dot(*normal) * normal).normalize()
}

fn get_color_from_raycast(ray: &Ray, object_list: &Vec<Box<dyn Intersectable>>, light_list: &Vec<Box<dyn Light>>, depth: u32) -> Vec3 {
    let mut hit_color = Vec3::new(0.0, 0.0, 0.0);

    if depth > MAX_RAY_DEPTH {
        return BACKGROUND_COLOR
    }

    let (collision_object, distance_parameter, mesh_info) = find_ray_intersection_with_scene(ray, object_list);
    if let Some(object) = collision_object {
        let intersection_point = ray.get_origin() + ray.get_direction() * distance_parameter;
        let normal = object.get_normal_at_intersection(&intersection_point, &mesh_info);
        let material_type = object.get_material_at_intersection(&intersection_point, &mesh_info).get_material_type();

        match material_type {
            MaterialType::Phong { diffuse_component, specular_component, power_component } => {
                let color_at_intersection = object.get_color_at_intersection(&intersection_point, &mesh_info);

                for light in light_list {

                    let (light_direction, attenuated_light, _light_distance_parameter) = light.get_light_direction_intensity_and_distance_parameter(intersection_point);

                    let shadow_ray = Ray::new(intersection_point - light_direction * math::NITRORAY_FLOAT_EPSILON, -light_direction);
                    let (shadow_collision_object, _shadow_parameter, _mesh_info) = find_ray_intersection_with_scene(&shadow_ray, object_list);
                    if shadow_collision_object.is_none() {
                        let diffuse = color_at_intersection * attenuated_light * f32::max(0.0, normal.dot(-light_direction));

                        let reflection_vector = get_reflection_vector(&light_direction, &normal);
                        let specular = attenuated_light * f32::powf(f32::max(0.0, reflection_vector.dot(-ray.get_direction())), power_component);

                        hit_color += diffuse * diffuse_component + specular * specular_component;
                    }
                }
            },
            MaterialType::Reflect => {
                let reflection_vector = get_reflection_vector(&ray.get_direction(), &normal);
                hit_color += REFLECTION_DIM_FACTOR * get_color_from_raycast(&Ray::new(intersection_point + reflection_vector * math::NITRORAY_FLOAT_EPSILON, reflection_vector), object_list, light_list, depth + 1);
            },
            MaterialType::ReflectRefract { refraction_component } => {
                let mut refraction_color = Vec3::splat(0.0);

                let reflection_mix = compute_fresnel(&ray.get_direction(), &normal, refraction_component);

                if reflection_mix < 1.0 {
                    let refraction_direction = get_refraction_vector(&ray.get_direction(), &normal, refraction_component);
                    refraction_color = get_color_from_raycast(&Ray::new(intersection_point + refraction_direction * math::NITRORAY_FLOAT_EPSILON, refraction_direction), object_list, light_list, depth + 1);
                }

                let reflection_direction = get_reflection_vector(&ray.get_direction(), &normal);
                let reflection_color = get_color_from_raycast(&Ray::new(intersection_point + reflection_direction * math::NITRORAY_FLOAT_EPSILON, reflection_direction), object_list, light_list, depth + 1);

                hit_color += reflection_color * reflection_mix + refraction_color * (1.0 - reflection_mix);
            },
        };
        
        return hit_color;
    }

    BACKGROUND_COLOR
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
    let camera = Camera::new(Vec3::new(0.0, 0.0, -3.0), 0.0, 0.0, 90.0, aspect_ratio);

    let plane_model = Rc::new(Model::load_model("res/models/plane.gltf"));

    let mut light_list: Vec<Box<dyn Light>> = Vec::new();
    light_list.push(Box::new(DirectionalLight::new(Vec3::new(-1.0, -1.0, 1.0).normalize(), Vec3::new(1.0, 1.0, 1.0), 1.0)));

    let red_material = Material::new(Vec3::new(1.0, 0.0, 0.0), MaterialType::Phong { diffuse_component: 1.0, specular_component: 0.0, power_component: 0.0 });
    let green_material = Material::new(Vec3::new(0.0, 1.0, 0.0), MaterialType::Phong { diffuse_component: 0.5, specular_component: 0.5, power_component: 0.5 });
    let blue_material = Material::new(Vec3::new(0.0, 0.0, 1.0), MaterialType::Phong { diffuse_component: 1.0, specular_component: 0.0, power_component: 0.0 });
    let purple_material = Material::new(Vec3::new(1.0, 0.0, 1.0), MaterialType::Phong { diffuse_component: 1.0, specular_component: 0.0, power_component: 0.0 });
    let yellow_material = Material::new(Vec3::new(1.0, 1.0, 0.0), MaterialType::Phong { diffuse_component: 1.0, specular_component: 0.0, power_component: 0.0 });
    let reflection_material = Material::new(Vec3::new(1.0, 1.0, 1.0), MaterialType::Reflect);
    let refraction_material = Material::new(Vec3::new(1.0, 1.0, 1.0), MaterialType::ReflectRefract { refraction_component: 0.5 });

    let mut object_list: Vec<Box<dyn Intersectable>> = Vec::new();
    object_list.push(Box::new(Sphere::new(Vec3::new(0.0, 0.0, 1.0), 1.0, red_material)));
    object_list.push(Box::new(Sphere::new(Vec3::new(0.0, 0.0, 0.0), 0.5, refraction_material)));
    object_list.push(Box::new(Sphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0, green_material)));
    object_list.push(Box::new(Sphere::new(Vec3::new(0.0, 3.0, 2.0), 1.0, blue_material)));
    object_list.push(Box::new(Sphere::new(Vec3::new(-2.0, 0.0, 1.0), 1.0, purple_material)));
    object_list.push(Box::new(ModelObject::new(
        Transform3d::new(Vec3::new(0.0, -1.0, -2.0), 0.0, 0.0, 0.0, Vec3::splat(2.0)), 
        yellow_material, 
        plane_model.clone()
    )));
    object_list.push(Box::new(ModelObject::new(
        Transform3d::new(Vec3::new(-4.0, 0.0, 0.0), -90.0, -45.0, 0.0, Vec3::splat(2.0)), 
        reflection_material, 
        plane_model.clone()
    )));

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

            framebuffer[x+y*WIDTH] = get_color_from_raycast(&ray, &object_list, &light_list, 0);
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