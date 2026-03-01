use xenofrost::core::math::Vec3;

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

pub(crate) struct Sphere {
    origin: Vec3,
    radius: f32,
    color: Vec3
}

impl Sphere {
    pub(crate) fn new(origin: Vec3, radius: f32, color: Vec3) -> Self {
        Self {
            origin,
            radius,
            color
        }
    }
}

impl Intersectable for Sphere {
    fn get_color(&self) -> Vec3 {
        self.color
    }

    fn intersect(&self, ray: &Ray) -> IntersectionInfo {
        let ray_origin_to_sphere_center = self.origin - ray.get_origin();
        let distance_squared = ray_origin_to_sphere_center.dot(ray_origin_to_sphere_center);

        let outside_sphere = distance_squared >= self.radius*self.radius;

        let closest_approach_parameter = ray_origin_to_sphere_center.dot(ray.get_direction());

        // Ray is outside of the sphere and pointing away from the sphere
        if outside_sphere && closest_approach_parameter < 0.0 {
            return IntersectionInfo { does_intersect: false, intersection_parameter: f32::INFINITY }
        }

        // Finds the distance from the center of the sphere to the point on the ray at which the closest intersection can occur
        let half_chord_distance_squared = (self.radius*self.radius) - distance_squared + (closest_approach_parameter * closest_approach_parameter);

        if outside_sphere && half_chord_distance_squared < 0.0 {
            return IntersectionInfo { does_intersect: false, intersection_parameter: f32::INFINITY }
        }

        let intersection_parameter = if outside_sphere {
            closest_approach_parameter - half_chord_distance_squared.sqrt()
        } else {
            closest_approach_parameter + half_chord_distance_squared.sqrt()
        };

        IntersectionInfo { does_intersect: true, intersection_parameter }
    }
    
    fn get_normal_at_intersection(&self, intersection_point: &Vec3) -> Vec3 {
        let normal = (intersection_point - self.origin).normalize();

        normal
    }
}