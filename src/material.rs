use xenofrost::core::math::Vec3;

#[derive(Clone, Copy)]
pub(crate) struct Material {
    base_color: Vec3,
    material_type: MaterialType
}

impl Material {
    pub(crate) fn new(base_color: Vec3, material_type: MaterialType) -> Self {
        Material { 
            base_color, 
            material_type 
        }
    }

    pub(crate) fn get_base_color(&self) -> Vec3 {
        self.base_color
    }

    pub(crate) fn get_material_type(&self) -> MaterialType {
        self.material_type
    }
}

#[derive(Clone, Copy)]
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

