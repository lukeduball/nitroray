use core::f32;

use xenofrost::core::math::Vec3;

use crate::{mesh::Mesh, object::{FaceIndex, IntersectionInfo}, ray::Ray};

pub(crate) struct Model {
    meshes: Vec<Mesh>
}

impl Model {
    pub(crate) fn load_model(model_name: &str) -> Self {
        let mut meshes = Vec::new();

        let gltf_result = gltf::import(model_name);

        match gltf_result {
            Ok((document, buffers, _images)) => {
                for mesh in document.meshes() {
                    meshes.push(Mesh::create_mesh(&mesh, &buffers));
                }
            },
            Err(e) => println!("Failed to load {} because of {}", model_name, e),
        }
        
        Self {meshes}
    }

    pub(crate) fn intersect(&self, local_ray: &Ray) -> IntersectionInfo {
        let mut does_intersect = false;
        let mut intersection_parameter = f32::INFINITY;
        let mut mesh_info = None;

        for (index, mesh) in self.meshes.iter().enumerate() {
            let intersection_info = mesh.intersect(local_ray);
            if intersection_info.does_intersect && intersection_info.intersection_parameter < intersection_parameter {
                does_intersect = true;
                intersection_parameter = intersection_info.intersection_parameter;
                mesh_info = Some(FaceIndex {
                    mesh_index: index as u32,
                    face_index: intersection_info.mesh_info.unwrap().face_index
                })
            }
        }

        IntersectionInfo { does_intersect, intersection_parameter, mesh_info }
    }

    pub(crate) fn get_normals_from_mesh_face(&self, mesh_index: u32, face_index: u32) -> Vec3 {
        self.meshes[mesh_index as usize].get_normals_of_face(face_index)
    }
}