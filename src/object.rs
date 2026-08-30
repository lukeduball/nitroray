use std::rc::Rc;

use xenofrost::core::math::{Mat4, Vec2, Vec3};

use crate::{material::Material, math::Transform3d, model::Model, ray::Ray};

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
    fn get_color_at_intersection(&self, intersection_point: &Vec3, mesh_info: &Option<FaceIndex>) -> Vec3;
    fn get_normal_at_intersection(&self, intersection_point: &Vec3, mesh_info: &Option<FaceIndex>) -> Vec3;
    fn get_texture_coords_at_intersection(&self, intersection_point: &Vec3, mesh_info: &Option<FaceIndex>) -> Vec2;
    fn get_material_at_intersection(&self, intersection_point: &Vec3, mesh_info: &Option<FaceIndex>) -> Rc<Material>;
}

pub(crate) struct ModelObject {
    transform: Transform3d,
    material: Rc<Material>,
    model: Rc<Model>,
}

impl ModelObject {
    pub(crate) fn new(transform: Transform3d, material: Rc<Material>, model: Rc<Model>) -> Self {
        Self {
            transform,
            material,
            model,
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

    fn get_normal_at_intersection(&self, _intersection_point: &Vec3, mesh_info: &Option<FaceIndex>) -> Vec3 {
        let mesh_face_indices = mesh_info.as_ref().unwrap();
        let local_normal = self.model.get_normals_from_mesh_face(mesh_face_indices.mesh_index, mesh_face_indices.face_index);
        let transformation_matrix = Mat4::from_scale_rotation_translation(self.transform.get_scale(), self.transform.get_rotation_quaternion(), self.transform.get_translation());
        transformation_matrix.transform_vector3(local_normal).normalize()
    }
    
    fn get_color_at_intersection(&self, intersection_point: &Vec3, mesh_info: &Option<FaceIndex>) -> Vec3 {
        let texture_coordinates = self.get_texture_coords_at_intersection(intersection_point, mesh_info);
        self.material.get_color_at_uv(texture_coordinates)
    }
    
    fn get_texture_coords_at_intersection(&self, intersection_point: &Vec3, mesh_info: &Option<FaceIndex>) -> Vec2 {
        let mesh_face_indices = mesh_info.as_ref().unwrap();
        let local_intersection_point = get_local_intersection(&self.transform, intersection_point);
        self.model.get_uv_coordinates_from_mesh_face_point(mesh_face_indices.mesh_index, mesh_face_indices.face_index, &local_intersection_point)
    }
    
    fn get_material_at_intersection(&self, _intersection_point: &Vec3, _mesh_info: &Option<FaceIndex>) -> Rc<Material> {
        self.material.clone()
    }
}

fn get_local_intersection(transform3d: &Transform3d, world_intersection_point: &Vec3) -> Vec3 {
    let transformation_matrix = Mat4::from_scale_rotation_translation(transform3d.get_scale(), transform3d.get_rotation_quaternion(), transform3d.get_translation());
    let inverse_transformation_matrix = transformation_matrix.inverse();
    inverse_transformation_matrix.transform_point3(*world_intersection_point)
}