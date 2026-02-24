use xenofrost::core::math::Vec3;

use crate::ray::Ray;

pub(crate) trait Intersectable {
    fn intersect(&self, ray: Ray) -> bool;
}

pub(crate) struct Sphere {
    origin: Vec3,
    radius: f32
}

impl Sphere {
    pub(crate) fn new(origin: Vec3, radius: f32) -> Self {
        Self {
            origin,
            radius
        }
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: Ray) -> bool {
        let ray_origin_to_sphere_center = self.origin - ray.get_origin();
        let distance_squared = ray_origin_to_sphere_center.dot(ray_origin_to_sphere_center);

        let outside_sphere = distance_squared >= self.radius*self.radius;

        let closest_approach_parameter = ray_origin_to_sphere_center.dot(ray.get_direction());

        // Ray is outside of the sphere and pointing away from the sphere
        if outside_sphere && closest_approach_parameter < 0.0 {
            return false
        }

        // Finds the distance from the center of the sphere to the point on the ray at which the closest intersection can occur
        let half_chord_distance_squared = (self.radius*self.radius) - distance_squared + (closest_approach_parameter * closest_approach_parameter);

        if outside_sphere && half_chord_distance_squared < 0.0 {
            return false
        }

        true
    }
}