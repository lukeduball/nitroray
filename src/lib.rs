use std::{sync::Arc, thread, time::Instant};

use image::{Rgb, RgbImage};
use xenofrost::core::math::Vec3;

use crate::{light::Light, material::MaterialType, object::{FaceIndex, Intersectable}, ray::Ray, scene::Scene};

mod camera;
mod data_structure;
mod geometry;
mod image_loader;
mod light;
mod material;
mod math;
mod mesh;
mod model;
mod object;
mod ray;
mod scene;

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
    let critical_value = 1.0 - refraction_component_ratio * refraction_component_ratio * (1.0 - incident_normal_cos * incident_normal_cos);

    (refraction_component_ratio * incident_direction + (refraction_component_ratio * incident_normal_cos - critical_value.sqrt()) * refraction_normal).normalize()
}

fn get_reflection_vector(incident_direction: &Vec3, normal: &Vec3) -> Vec3 {
    (incident_direction - 2.0 * incident_direction.dot(*normal) * normal).normalize()
}

fn get_color_from_raycast(ray: &Ray, object_list: &Vec<Arc<dyn Intersectable + Sync + Send>>, light_list: &Vec<Arc<dyn Light + Sync + Send>>, depth: u32) -> Vec3 {
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
                    else {
                        //Add some ambient lighting
                        hit_color += color_at_intersection * 0.1;
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

fn find_ray_intersection_with_scene<'a>(ray: &'a Ray, object_list: &'a Vec<Arc<dyn Intersectable + Sync + Send>>) -> (Option<&'a Arc<dyn Intersectable + Sync + Send>>, f32, Option<FaceIndex>) {
    let mut min_distance_parameter = f32::INFINITY;
    let mut collision_object: Option<&Arc<dyn Intersectable + Sync + Send>> = None;
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

struct SendPtr(*mut Vec3);
unsafe impl Send for SendPtr {}

pub fn run() {
    let scene_result = Scene::load_scene("res/scenes/scene.json");
    match scene_result {
        Ok(scene) => {
            let start = Instant::now();

            let mut framebuffer = vec![Vec3::new(0.0, 0.0, 0.0); scene.image_width as usize*scene.image_height as usize];
            let aspect_ratio = scene.image_width as f32 / scene.image_height as f32;
            let field_of_view_component = f32::tan(scene.camera.get_field_of_view() / 2.0);

            let scene_resource = Arc::new(scene);

            match thread::available_parallelism() {
                Ok(cores) => {
                    let num_threads = cores.get();
                    println!("Number of available cores: {}", num_threads);

                    let workgroup_width = scene_resource.image_width / 4 as u32;
                    let workgroup_height = scene_resource.image_height / 3 as u32;

                    for i in 0..num_threads {
                        thread::scope(|s| {
                            let base_ptr = framebuffer.as_mut_ptr();
                            let wrapper = SendPtr(base_ptr);

                            let scene_resource_clone = scene_resource.clone();
                            
                            s.spawn(move || {
                                let raw_pointer = wrapper;

                                let x_modifier = i % 4;
                                let y_modifier = i / 4;
                                for x in 0..workgroup_width {
                                    let frame_x = x_modifier as u32 * workgroup_width + x;
                                    for y in 0..workgroup_height{
                                        let frame_y = y_modifier as u32 * workgroup_height + y;

                                        if frame_x < scene_resource_clone.image_width && frame_y < scene_resource_clone.image_height {
                                            //pixel screen gets the center of each pixel and divides to put it in normalized coordinates between 0 and 1
                                            let pixel_screen_x = (frame_x as f32 + 0.5) / scene_resource_clone.image_width as f32;
                                            let pixel_screen_y = (frame_y as f32 + 0.5) / scene_resource_clone.image_height as f32;
                                            let pixel_camera_x = (2.0 * pixel_screen_x - 1.0) * aspect_ratio * field_of_view_component;
                                            let pixel_camera_y = (1.0 - 2.0 * pixel_screen_y) * field_of_view_component;
                                            let pixel_coordinate = Vec3::new(pixel_camera_x, pixel_camera_y, 1.0);
                                            let world_coordinate = scene_resource_clone.camera.convert_view_space_to_world_space(pixel_coordinate);
                                            let ray_direction = (world_coordinate - scene_resource_clone.camera.get_origin()).normalize();
                                            let ray = Ray::new(world_coordinate, ray_direction);

                                            unsafe {
                                                (*raw_pointer.0.offset(frame_x as isize + frame_y as isize * scene_resource_clone.image_width as isize)) = get_color_from_raycast(&ray, &scene_resource_clone.object_list, &scene_resource_clone.light_list, 0);
                                            }
                                        }
                                    }
                                }
                            });
                        });
                    }
                },
                Err(_) => {
                    eprintln!("Unable to get number of cores!")
                },
            }

            let mut out_image = RgbImage::new(scene_resource.image_width, scene_resource.image_height as u32);
            for x in 0..scene_resource.image_width {
                for y in 0..scene_resource.image_height {
                    let index = x + y*scene_resource.image_width;
                    let pixel = framebuffer[index as usize] * 255.0;
                    let red = pixel.x as u8;
                    let green = pixel.y as u8;
                    let blue = pixel.z as u8;
                    out_image.put_pixel(x as u32, y as u32, Rgb([red, green, blue]));
                } 
            }

            let duration = start.elapsed();

            println!("Ray Tracing took {} ms", duration.as_millis());

            let _ = out_image.save("res/out.png");
        },
        Err(error) => eprintln!("Error Encountered: {}", error),
    }
}