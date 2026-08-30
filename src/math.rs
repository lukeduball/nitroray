use std::mem::swap;

use serde::Deserialize;
use xenofrost::core::math::{EulerRot, Quat, Vec3};

use crate::ray::Ray;

pub(crate) const NITRORAY_FLOAT_EPSILON: f32 = 0.0001;

#[derive(Deserialize)]
pub(crate) struct Transform3d {
    translation: Vec3,
    pitch: f32,
    yaw: f32,
    roll: f32,
    scale: Vec3
}

impl Transform3d {
    pub(crate) fn get_translation(&self) -> Vec3 {
        self.translation
    }

    pub(crate) fn get_rotation_quaternion(&self) -> Quat {
        Quat::from_euler(EulerRot::YXZ, self.yaw.to_radians(), self.pitch.to_radians(), self.roll.to_radians())
    }

    pub(crate) fn get_scale(&self) -> Vec3 {
        self.scale
    }
}

pub(crate) fn get_direction_vector_from_yaw_and_pitch(yaw: f32, pitch: f32) -> Vec3 {
    let x = f32::sin(yaw.to_radians()) * f32::cos(pitch.to_radians());
    let y = f32::sin(pitch.to_radians());
    let z = f32::cos(yaw.to_radians()) * f32::cos(pitch.to_radians());
    Vec3::new(x, y, z).normalize()
}

pub(crate) fn are_floats_equal(f1: f32, f2: f32) -> bool {
    (f1 - f2).abs() < NITRORAY_FLOAT_EPSILON
}

#[derive(Clone, Copy)]
pub(crate) struct AxisAlignedBoundingBox {
    center: Vec3,
    half_distances: Vec3
}

impl AxisAlignedBoundingBox {
    pub(crate) fn new_from_points(min_point: Vec3, max_point: Vec3) -> AxisAlignedBoundingBox {
        Self::new(
            (min_point + max_point) / 2.0, 
            (max_point - min_point) / 2.0
        )
    }

    pub(crate) fn new(center: Vec3, half_distances: Vec3) -> AxisAlignedBoundingBox {
        Self {
            center,
            half_distances
        }
    }

    pub(crate) fn get_center(&self) -> Vec3 {
        self.center
    }

    pub(crate) fn get_half_distances(&self) -> Vec3 {
        self.half_distances
    }

    pub fn intersect_ray(&self, ray: &Ray) -> (bool, f32) {
        //TODO this function can be optimized. The divide by zero needs to be evaluated to make sure it doesn't cause issues if both -INF or +INF are the result

        let minimum_point = self.center - self.half_distances;
        let maximum_point = self.center + self.half_distances;

        let mut t_x_min = (minimum_point.x - ray.get_origin().x) / ray.get_direction().x;
        let mut t_x_max = (maximum_point.x - ray.get_origin().x) / ray.get_direction().x;

        if t_x_min > t_x_max {
            std::mem::swap(&mut t_x_min, &mut t_x_max);
        }

        let mut t_y_min = (minimum_point.y - ray.get_origin().y) / ray.get_direction().y;
        let mut t_y_max = (maximum_point.y - ray.get_origin().y) / ray.get_direction().y;

        if t_y_min > t_y_max {
            std::mem::swap(&mut t_y_min, &mut t_y_max);
        }

        if (t_x_min > t_y_max) || (t_y_min > t_x_max) {
            return (false, f32::INFINITY)
        }

        let t_xy_min = t_x_min.max(t_y_min);
        let t_xy_max = t_x_max.min(t_y_max);

        let mut t_z_min = (minimum_point.z - ray.get_origin().z) / ray.get_direction().z;
        let mut t_z_max = (maximum_point.z - ray.get_origin().z) / ray.get_direction().z;

        if t_z_min > t_z_max {
            std::mem::swap(&mut t_z_min, &mut t_z_max);
        }

        if (t_xy_min > t_z_max) || (t_z_min > t_xy_max) {
            return (false, f32::INFINITY)
        }

        let t_min = t_xy_min.max(t_z_min);
        let t_max = t_xy_max.min(t_z_max);

        let t = if t_min > 0.0 {
            t_min
        } else {
            t_max
        };

        if t < 0.0 {
            return (false, f32::INFINITY);
        }


        (true, t)
    }

    pub fn is_triangle_overlapping(&self, vertex1: &Vec3, vertex2: &Vec3, vertex3: &Vec3) -> bool {
        //Translate the bounding box to the origin
        let transformed_vertex1 = vertex1 - self.center;
        let transformed_vertex2 = vertex2 - self.center;
        let transformed_vertex3 = vertex3 - self.center;

        //Calculate the edges of the triangle
        let edge21 = transformed_vertex2 - transformed_vertex1;
        let edge32 = transformed_vertex3 - transformed_vertex2;
        let edge13 = transformed_vertex1 - transformed_vertex3;


        //Test if the box intersects the plane of the triangle
        let triangle_normal = edge21.cross(edge32).normalize();
        let plane_constant = -transformed_vertex2.dot(triangle_normal);

        if !self.does_plane_intersect(&triangle_normal, plane_constant) {
            return false;
        }

        //Test the minimal AABB around the triangle projected onto each normal of the cube (1,0,0), (0, 1,0) and (0,0,1)
        for i in 0..3 {
            let min = transformed_vertex1[i].min(transformed_vertex2[i]).min(transformed_vertex3[i]);
            let max = transformed_vertex1[i].max(transformed_vertex2[i]).max(transformed_vertex3[i]);
            if min > self.half_distances[i] || max < -self.half_distances[i] {
                return false;
            }
        }

        //Perform tests with edge21 which project the triangle onto and axis and the box onto the axis
        let edge21abs = edge21.abs();
        if !Self::axis_test(edge21.z, edge21.y, transformed_vertex1.y, transformed_vertex1.z, transformed_vertex3.y, transformed_vertex3.z, edge21abs.z, self.half_distances.y, edge21abs.y, self.half_distances.z) {
            return false;
        }
        //Y axis
        if !Self::axis_test(edge21.x, edge21.z, transformed_vertex1.z, transformed_vertex1.x, transformed_vertex3.z, transformed_vertex3.x, edge21abs.z, self.half_distances.x, edge21abs.x, self.half_distances.z) {
            return false;
        }
        //Z axis
        if !Self::axis_test(edge21.y, edge21.x, transformed_vertex2.x, transformed_vertex2.y, transformed_vertex3.x, transformed_vertex3.y, edge21abs.y, self.half_distances.x, edge21abs.x, self.half_distances.y) {
            return false;
        }

        //Perform tests with edge32 which project the triangle onto and axis and the box onto the axis
        let edge32abs = edge32.abs();
        //X axis
        if !Self::axis_test(edge32.z, edge32.y, transformed_vertex1.y, transformed_vertex1.z, transformed_vertex3.y, transformed_vertex3.z, edge32abs.z, self.half_distances.y, edge32abs.y, self.half_distances.z) {
            return false;
        }
        //Y axis
        if !Self::axis_test(edge32.x, edge32.z, transformed_vertex1.z, transformed_vertex1.x, transformed_vertex3.z, transformed_vertex3.x, edge32abs.z, self.half_distances.x, edge32abs.x, self.half_distances.z) { 
            return false;
        }
        //Z axis
        if !Self::axis_test(edge32.y, edge32.x, transformed_vertex1.x, transformed_vertex1.y, transformed_vertex2.x, transformed_vertex2.y, edge32abs.y, self.half_distances.x, edge32abs.x, self.half_distances.y) { 
            return false;
        }
        
        //Perform tests with edge13 which project the triangle onto and axis and the box onto the axis
        let edge13abs = edge13.abs();
        //X axis
        if !Self::axis_test(edge13.z, edge13.y, transformed_vertex1.y, transformed_vertex1.z, transformed_vertex2.y, transformed_vertex2.z, edge13abs.z, self.half_distances.y, edge13abs.y, self.half_distances.z) { 
            return false;
        }
        //Y axis
        if !Self::axis_test(edge13.x, edge13.z, transformed_vertex1.z, transformed_vertex1.x, transformed_vertex2.z, transformed_vertex2.x, edge13abs.z, self.half_distances.x, edge13abs.x, self.half_distances.z) {
            return false;
        }
        //Z axis
        if !Self::axis_test(edge13.y, edge13.x, transformed_vertex2.x, transformed_vertex2.y, transformed_vertex3.x, transformed_vertex3.y, edge13abs.y, self.half_distances.x, edge13abs.x, self.half_distances.y) {
            return false;
        }

        true
    }

    pub fn does_plane_intersect(&self, plane_normal: &Vec3, plane_constant: f32) -> bool {
        let extent = self.half_distances.dot(plane_normal.abs());
        //This is just the plane constant because the box was transformed to be at (0,0,0) and the dot of (0,0,0) with anything is zero
        let signed_distance = plane_constant;

        if signed_distance - extent > 0.0 {
            return false;
        }
        if signed_distance + extent < 0.0 {
            return false;
        }
        
        true
    }

    fn axis_test(edge1: f32, edge2: f32, v00: f32, v01: f32, v10: f32, v11: f32, abs_edge0: f32, box0: f32, abs_edge1: f32, box1: f32) -> bool {
        //Projection of the triangle onto the axis
        let mut max = edge1 * v00 - edge2 * v01;
        let mut min = edge1 * v10 - edge2 * v11;

        if max < min
        {
            swap(&mut min, &mut max);
        }

        //Projection of the box onto the axis
        let radius = abs_edge0 * box0 + abs_edge1 * box1;

        //If the min point of the projected triangle is greater than the radius or max point is less than the radius of the projected box, they do not overlap in that axis
        if min > radius || max < -radius
        {
            return false;
        }
        return true;
    }
}