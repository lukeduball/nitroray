use core::f32;

use xenofrost::core::math::{Mat4, Vec3};

use crate::math::are_floats_equal;

pub(crate) struct Ray {
    origin: Vec3,
    direction: Vec3
}

impl Ray {
    pub(crate) fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction
        }
    }

    pub(crate) fn convert_ray_to_another_space(&self, transformation_matrix: &Mat4) -> Self {
        let origin = transformation_matrix.transform_point3(self.origin);
        let direction = transformation_matrix.transform_vector3(self.direction);
        
        Self {
            origin,
            direction
        }
    }

    pub(crate) fn convert_parameter_to_another_space(&self, local_parameter: f32, other_space_ray: &Ray, transformation_matrix: &Mat4) -> f32 {
        let local_intersection = self.origin + self.direction * local_parameter;
        let other_space_intersection = transformation_matrix.transform_point3(local_intersection);
        let other_space_ray_distance_to_intersection = other_space_intersection - other_space_ray.origin;

        for axis in 0..3 {
            if !are_floats_equal(other_space_ray.direction[axis], 0.0) {
                return other_space_ray_distance_to_intersection[axis] / other_space_ray.direction[axis];
            }
        }

        return f32::INFINITY
    }

    pub(crate) fn get_origin(&self) -> Vec3 {
        self.origin
    }

    pub(crate) fn get_direction(&self) -> Vec3 {
        self.direction
    }
}