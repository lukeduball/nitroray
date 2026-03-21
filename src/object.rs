use std::rc::Rc;

use xenofrost::core::math::{Mat4, Vec3};

use crate::{math::Transform3d, model::Model, ray::Ray};

pub(crate) struct FaceIndex {
    pub(crate) mesh_index: u32,
    pub(crate) face_index: u32
}

pub(crate) struct IntersectionInfo {
    pub(crate) does_intersect: bool,
    pub(crate) intersection_parameter: f32,
    pub(crate) mesh_info: Option<FaceIndex>
}

pub(crate) trait Intersectable {
    fn intersect(&self, ray: &Ray) -> IntersectionInfo;
    fn get_color(&self) -> Vec3;
    fn get_normal_at_intersection(&self, intersection_point: &Vec3, mesh_info: Option<FaceIndex>) -> Vec3;
}

pub(crate) struct ModelObject {
    transform: Transform3d,
    color: Vec3,
    model: Rc<Model>,
}

impl ModelObject {
    pub(crate) fn new(transform: Transform3d, color: Vec3, model: Rc<Model>) -> Self {
        Self {
            transform,
            color,
            model
        }
    }
}

impl Intersectable for ModelObject {
    fn intersect(&self, ray: &Ray) -> IntersectionInfo {
        let transformation_matrix = Mat4::from_scale_rotation_translation(self.transform.get_scale(), self.transform.get_rotation_quaternion(), self.transform.get_translation());
        let local_ray = ray.convert_ray_to_another_space(&transformation_matrix.inverse());

        let local_intersection_info = self.model.intersect(&local_ray);

        let world_ray_parameter = local_ray.convert_parameter_to_another_space(local_intersection_info.intersection_parameter, ray, &transformation_matrix);
        IntersectionInfo { 
            does_intersect: local_intersection_info.does_intersect, 
            intersection_parameter: world_ray_parameter, 
            mesh_info: local_intersection_info.mesh_info 
        }
    }

    fn get_color(&self) -> Vec3 {
        self.color
    }

    fn get_normal_at_intersection(&self, _intersection_point: &Vec3, mesh_info: Option<FaceIndex>) -> Vec3 {
        let mesh_face_indices = mesh_info.unwrap();
        self.model.get_normals_from_mesh_face(mesh_face_indices.mesh_index, mesh_face_indices.face_index)
    }
}