use std::{collections::HashMap, error::Error, fs::File, io::BufReader, rc::Rc};

use image::Rgb32FImage;
use serde::Deserialize;
use xenofrost::core::math::Vec3;

use crate::{camera::Camera, geometry::{Sphere, Triangle}, image_loader::ImageLoader, light::Light, material::{Material, MaterialType}, math::Transform3d, model::Model, object::{Intersectable, ModelObject}, light::DirectionalLight};

#[derive(Deserialize)]
struct CameraParser {
    position: [f32; 3],
    yaw: f32,
    pitch: f32,
    field_of_view: f32
}

#[derive(Deserialize)]
struct TriangleParser {
    vertex1: [f32; 3],
    vertex2: [f32; 3],
    vertex3: [f32; 3],
    material: String
}

#[derive(Deserialize)]
struct SphereParser {
    origin: [f32; 3],
    radius: f32,
    material: String
}

#[derive(Deserialize)]
struct Transform3dParser {
    translation: [f32; 3],
    pitch: f32,
    yaw: f32,
    roll: f32,
    scale: [f32; 3]
}

#[derive(Deserialize)]
struct ModelObjectParser {
    transform3d: Transform3dParser,
    material: String,
    model: String
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ObjectType {
    Triangle(TriangleParser),
    Sphere(SphereParser),
    Model(ModelObjectParser)
}

#[derive(Deserialize)]
struct MaterialParser {
    identifier: String,
    base_color: [f32; 3],
    material_type: MaterialType,
    texture: Option<String>
}

#[derive(Deserialize)]
struct DirectionalLightParser {
    direction: [f32; 3],
    color: [f32; 3],
    intensity: f32
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum LightType {
    DirectionalLight(DirectionalLightParser)
}

#[derive(Deserialize)]
struct SceneParser {
    image_dimensions: [u32; 2],
    materials: Vec<MaterialParser>,
    camera_info: CameraParser,
    objects: Vec<ObjectType>,
    lights: Vec<LightType>
}

pub(crate) struct Scene {
    pub image_width: u32,
    pub image_height: u32,
    pub camera: Camera,
    pub object_list: Vec<Box<dyn Intersectable>>,
    pub light_list: Vec<Box<dyn Light>>
}

impl Scene {
    pub(crate) fn load_scene(scene_file_name: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(scene_file_name)?;
        let reader = BufReader::new(file);

        let parsed_scene: SceneParser = serde_json::from_reader(reader)?;

        let aspect_ratio = parsed_scene.image_dimensions[0] as f32 / parsed_scene.image_dimensions[1] as f32;
        let camera = Camera::new(
            Vec3::from_array(parsed_scene.camera_info.position), 
            parsed_scene.camera_info.yaw, 
            parsed_scene.camera_info.pitch, 
            parsed_scene.camera_info.field_of_view, 
            aspect_ratio
        );

        let image_loader = ImageLoader::new();
        let mut materials_hashmap: HashMap<String, Rc<Material>> = HashMap::new();
        let mut images_hashmap: HashMap<String, Rc<Rgb32FImage>> = HashMap::new();

        for parsed_material in parsed_scene.materials {
            let material = match parsed_material.texture {
                Some(texture_name) => {
                    let texture = if images_hashmap.contains_key(&texture_name) {
                        images_hashmap.get(&texture_name).unwrap().clone()
                    } else {
                        let image = image_loader.load_image(format!("res/images/{texture_name}").as_str());
                        images_hashmap.insert(texture_name, image.clone());
                        image
                    };
                    Rc::new(Material::new_with_image(Vec3::from_array(parsed_material.base_color), parsed_material.material_type, texture))
                },
                None => Rc::new(Material::new(Vec3::from_array(parsed_material.base_color), parsed_material.material_type)),
            };
            materials_hashmap.insert(parsed_material.identifier, material);
        }

        let mut model_hashmap: HashMap<String, Rc<Model>> = HashMap::new();
        let mut object_list = Vec::new();

        for parsed_object in parsed_scene.objects {
            let object: Box<dyn Intersectable> = match parsed_object {
                ObjectType::Triangle(triangle_parser) => Box::new(
                    Triangle::new(
                        Vec3::from_array(triangle_parser.vertex1), 
                        Vec3::from_array(triangle_parser.vertex2), 
                        Vec3::from_array(triangle_parser.vertex3), 
                        materials_hashmap.get(&triangle_parser.material).unwrap().clone()
                    )
                ),
                ObjectType::Sphere(sphere_parser) => Box::new(
                    Sphere::new(
                        Vec3::from_array(sphere_parser.origin),
                        sphere_parser.radius,
                        materials_hashmap.get(&sphere_parser.material).unwrap().clone()
                    )
                ),
                ObjectType::Model(model_object) => {
                    let model = if model_hashmap.contains_key(&model_object.model) {
                        model_hashmap.get(&model_object.model).unwrap().clone()
                    } else {
                        let model_ref = Rc::new(Model::load_model(format!("res/models/{}", model_object.model).as_str()));
                        model_hashmap.insert(model_object.model, model_ref.clone());
                        model_ref
                    };

                    let transform3d = Transform3d::new(
                        Vec3::from_array(model_object.transform3d.translation), 
                        model_object.transform3d.pitch, 
                        model_object.transform3d.yaw, 
                        model_object.transform3d.roll, 
                        Vec3::from_array(model_object.transform3d.scale)
                    );

                    let material = materials_hashmap.get(&model_object.material).unwrap().clone();

                    Box::new(ModelObject::new(transform3d, material, model))
                },
            };

            object_list.push(object);
        }

        let mut light_list = Vec::new();

        for parsed_light in parsed_scene.lights {
            let light: Box<dyn Light> = match parsed_light {
                LightType::DirectionalLight(directional_light_parser) => Box::new(
                    DirectionalLight::new(
                        Vec3::from_array(directional_light_parser.direction), 
                        Vec3::from_array(directional_light_parser.color), 
                        directional_light_parser.intensity
                    )
                ),
            };

            light_list.push(light);
        }

        Ok(
            Self {
                image_width: parsed_scene.image_dimensions[0],
                image_height: parsed_scene.image_dimensions[1],
                camera,
                object_list,
                light_list
            }
        )
    }
}