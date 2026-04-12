use xenofrost::core::math::{EulerRot, Quat, Vec3};

use crate::ray::Ray;

pub(crate) const NITRORAY_FLOAT_EPSILON: f32 = 0.0001;

pub(crate) struct Transform3d {
    translation: Vec3,
    pitch: f32,
    yaw: f32,
    roll: f32,
    scale: Vec3
}

impl Transform3d {
    pub(crate) fn new(translation: Vec3, pitch: f32, yaw: f32, roll: f32, scale: Vec3) -> Self {
        Self {
            translation,
            pitch,
            yaw,
            roll,
            scale
        }
    }

    pub(crate) fn from_translation(translation: Vec3) -> Self {
        Self::new(translation, 0.0, 0.0, 0.0, Vec3::splat(1.0))
    }

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
}