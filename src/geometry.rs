use core::f32;

use xenofrost::core::math::{Vec2, Vec3};

use crate::{material::Material, math::are_floats_equal, object::{FaceIndex, Intersectable, IntersectionInfo}, ray::Ray};

pub(crate) struct Sphere {
    origin: Vec3,
    radius: f32,
    material: Material
}

impl Sphere {
    pub(crate) fn new(origin: Vec3, radius: f32, material: Material) -> Self {
        Self {
            origin,
            radius,
            material
        }
    }
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> IntersectionInfo {
        let ray_origin_to_sphere_center = self.origin - ray.get_origin();
        let distance_squared = ray_origin_to_sphere_center.dot(ray_origin_to_sphere_center);

        let outside_sphere = distance_squared >= self.radius*self.radius;

        let closest_approach_parameter = ray_origin_to_sphere_center.dot(ray.get_direction());

        // Ray is outside of the sphere and pointing away from the sphere
        if outside_sphere && closest_approach_parameter < 0.0 {
            return IntersectionInfo { does_intersect: false, intersection_parameter: f32::INFINITY, mesh_info: None }
        }

        // Finds the distance from the center of the sphere to the point on the ray at which the closest intersection can occur
        let half_chord_distance_squared = (self.radius*self.radius) - distance_squared + (closest_approach_parameter * closest_approach_parameter);

        if outside_sphere && half_chord_distance_squared < 0.0 {
            return IntersectionInfo { does_intersect: false, intersection_parameter: f32::INFINITY, mesh_info: None }
        }

        let intersection_parameter = if outside_sphere {
            closest_approach_parameter - half_chord_distance_squared.sqrt()
        } else {
            closest_approach_parameter + half_chord_distance_squared.sqrt()
        };

        IntersectionInfo { does_intersect: true, intersection_parameter, mesh_info: None }
    }
    
    fn get_normal_at_intersection(&self, intersection_point: &Vec3, _mesh_info: &Option<FaceIndex>) -> Vec3 {
        let normal = (intersection_point - self.origin).normalize();

        normal
    }
    
    fn get_texture_coords_at_intersection(&self, _intersection_point: &Vec3, _mesh_info: &Option<FaceIndex>) -> Option<Vec2> {
        None
    }
    
    fn get_material_at_intersection(&self, _intersection_point: &Vec3, _mesh_info: &Option<FaceIndex>) -> Material {
        self.material
    }
    
    fn get_color_at_intersection(&self, _intersection_point: &Vec3, _mesh_info: &Option<FaceIndex>) -> Vec3 {
        self.material.get_base_color()
    }
}

pub(crate) struct Triangle {
    vertices: [Vec3; 3],
    material: Material
}

impl Triangle {
    pub(crate) fn new(vertex1: Vec3, vertex2: Vec3, vertex3: Vec3, material: Material) -> Self {
        Self {
            vertices: [vertex1, vertex2, vertex3],
            material
        }
    }

    pub(crate) fn intersect_triangle(ray: &Ray, vertex1: &Vec3, vertex2: &Vec3, vertex3: &Vec3) -> IntersectionInfo {
        let side_v1_v2 = vertex2 - vertex1;
        let side_v1_v3 = vertex3 - vertex1;

        //Perform triple product to calculate the determinant
        let direction_edge_v1_v3_cross_product = ray.get_direction().cross(side_v1_v3);
        let determinant = side_v1_v2.dot(direction_edge_v1_v3_cross_product);

        //A determinant of zero indicates the ray and triangle are parallel
        if are_floats_equal(determinant, 0.0) {
            return IntersectionInfo { does_intersect: false, intersection_parameter: f32::INFINITY, mesh_info: None }
        }

        let inverse_determinant = 1.0 / determinant;

        //Find the normalized u coordinate by performing triple scalar product with ray direction vector, side_v1_v3, and the origin to v1
        let origin_v1_vector = ray.get_origin() - vertex1;
        let u = origin_v1_vector.dot(direction_edge_v1_v3_cross_product) * inverse_determinant;

        if u < 0.0 || u > 1.0 {
            //The intersection point with the plane is outside of the triangle
            return IntersectionInfo { does_intersect: false, intersection_parameter: f32::INFINITY, mesh_info: None }
        }

        //Find the normalized v coordinate by performing the triple scalar product with the origin to v1, side_v1_v2, and the direction vector of the ray
        let origin_edge_v1_v2_cross = origin_v1_vector.cross(side_v1_v2);
        let v = ray.get_direction().dot(origin_edge_v1_v2_cross) * inverse_determinant;

        if v < 0.0 || v + u > 1.0 {
            //The intersection point with the plane is outside of the triangle
            return IntersectionInfo { does_intersect: false, intersection_parameter: f32::INFINITY, mesh_info: None }
        }

        //Find the intersection parameter distance of the ray by performing the triple scalar product with origin to v1, side_v1_v2, and side_v1_v3
        let intersection_parameter = side_v1_v3.dot(origin_edge_v1_v2_cross) * inverse_determinant;

        if intersection_parameter < 0.0 {
            //The ray is facing away from the triangle so there is no intersection
            return IntersectionInfo { does_intersect: false, intersection_parameter: f32::INFINITY, mesh_info: None }
        }
        
        IntersectionInfo { does_intersect: true, intersection_parameter: intersection_parameter, mesh_info: None }
    } 
}

impl Intersectable for Triangle {
    fn intersect(&self, ray: &Ray) -> IntersectionInfo {
        Self::intersect_triangle(ray, &self.vertices[0], &self.vertices[1], &self.vertices[2])
    }

    fn get_normal_at_intersection(&self, _intersection_point: &Vec3, _mesh_info: &Option<FaceIndex>) -> Vec3 {
        ((self.vertices[1] - self.vertices[0]).cross(self.vertices[2] - self.vertices[0])).normalize()
    }
    
    fn get_color_at_intersection(&self, _intersection_point: &Vec3, _mesh_info: &Option<FaceIndex>) -> Vec3 {
        self.material.get_base_color()
    }
    
    fn get_texture_coords_at_intersection(&self, _intersection_point: &Vec3, _mesh_info: &Option<FaceIndex>) -> Option<Vec2> {
        None
    }
    
    fn get_material_at_intersection(&self, _intersection_point: &Vec3, _mesh_info: &Option<FaceIndex>) -> Material {
        self.material
    }
}