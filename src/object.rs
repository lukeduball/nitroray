use xenofrost::core::math::{Mat4, Vec3};

use crate::ray::Ray;

pub(crate) struct IntersectionInfo {
    pub(crate) does_intersect: bool,
    pub(crate) intersection_parameter: f32
}

pub(crate) trait Intersectable {
    fn intersect(&self, ray: &Ray) -> IntersectionInfo;
    fn get_color(&self) -> Vec3;
    fn get_normal_at_intersection(&self, intersection_point: &Vec3) -> Vec3;
}

pub(crate) trait Object {
    fn get_position(&self) -> Vec3;
    fn get_pitch(&self) -> f32;
    fn get_yaw(&self) -> f32;
    fn get_roll(&self) -> f32;
    fn get_scale(&self) -> Vec3;
    fn get_world_to_local_transform(&self) -> Mat4;
    fn get_local_to_world_transform(&self) -> Mat4;
}