use std::rc::Rc;

use image::Rgb32FImage;
use serde::Deserialize;
use xenofrost::core::math::Vec3;

#[derive(Clone)]
pub(crate) struct Material {
    base_color: Vec3,
    material_type: MaterialType,
    texture: Option<Rc<Rgb32FImage>>
}

impl Material {
    pub(crate) fn new(base_color: Vec3, material_type: MaterialType) -> Self {
        Self { 
            base_color, 
            material_type,
            texture: None 
        }
    }

    pub(crate) fn new_with_image(base_color: Vec3, material_type: MaterialType, texture: Rc<Rgb32FImage>) -> Self {
        Self {
            base_color,
            material_type,
            texture: Some(texture)
        }
    }

    pub(crate) fn get_texture(&self) -> Option<Rc<Rgb32FImage>> {
        self.texture.clone()
    }

    pub(crate) fn get_base_color(&self) -> Vec3 {
        self.base_color
    }

    pub(crate) fn get_material_type(&self) -> MaterialType {
        self.material_type
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(tag = "type")]
pub(crate) enum MaterialType {
    Phong {
        diffuse_component: f32,
        specular_component: f32,
        power_component: f32
    },
    Reflect,
    ReflectRefract {
        refraction_component: f32
    }
}

