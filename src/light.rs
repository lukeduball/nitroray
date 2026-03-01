use core::f32;

use xenofrost::core::math::Vec3;

pub trait Light {
    fn get_light_direction_intensity_and_distance_parameter(&self, intersection_point: Vec3) -> (Vec3, Vec3, f32);
}

pub struct DirectionalLight {
    direction: Vec3,
    color: Vec3,
    intensity: f32
}

impl DirectionalLight {
    pub fn new(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            direction,
            color,
            intensity
        }
    }
}

impl Light for DirectionalLight {
    fn get_light_direction_intensity_and_distance_parameter(&self, _intersection_point: Vec3) -> (Vec3, Vec3, f32) {
        let light_direction = self.direction;
        let attenuated_light = self.color * self.intensity;
        let distance_parameter = f32::INFINITY;

        (light_direction, attenuated_light, distance_parameter)
    }
}