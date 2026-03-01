use xenofrost::core::math::{Mat4, Vec3};

use crate::math::get_direction_vector_from_yaw_and_pitch;

pub(crate) struct Camera {
    origin: Vec3,
    field_of_view: f32,
    _view_matrix: Mat4,
    inverse_view_matrix: Mat4,
    _projection_matrix: Mat4
}

impl Camera {
    pub(crate) fn new(origin: Vec3, yaw: f32, pitch: f32, field_of_view: f32, aspect_ratio: f32) -> Self {
        let front_direction = get_direction_vector_from_yaw_and_pitch(yaw, pitch);
        let view_matrix = Mat4::look_to_lh(origin, front_direction, Vec3::new(0.0, 1.0, 0.0));
        let inverse_view_matrix = view_matrix.inverse();
        let projection_matrix = Mat4::perspective_lh(field_of_view.to_radians(), aspect_ratio, 0.1, 100.0);
        Self {
            origin,
            field_of_view,
            _view_matrix: view_matrix,
            inverse_view_matrix,
            _projection_matrix: projection_matrix
        }
    }

    pub(crate) fn convert_view_space_to_world_space(&self, view_coordinate: Vec3) -> Vec3 {
        self.inverse_view_matrix.transform_point3(view_coordinate)
    }

    pub(crate) fn get_field_of_view(&self) -> f32 {
        self.field_of_view.to_radians()
    }

    pub(crate) fn get_origin(&self) -> Vec3 {
        self.origin
    }
}