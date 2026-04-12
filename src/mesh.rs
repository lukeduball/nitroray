use core::f32;

use xenofrost::core::math::{Vec2, Vec3};

use crate::{geometry::Triangle, math::AxisAlignedBoundingBox, object::{FaceIndex, IntersectionInfo}, ray::Ray};

struct Face {
    indices: [u32; 3],
}

pub(crate) struct Mesh {
    faces: Vec<Face>,
    vertices: Vec<Vec3>,
    texture_coords: Vec<Vec2>,
    normals: Option<Vec<Vec3>>,
    axis_aligned_bounding_box: AxisAlignedBoundingBox
}

impl Mesh {
    pub(crate) fn create_mesh(mesh: &gltf::Mesh, buffers: &Vec<gltf::buffer::Data>) -> Self {
        let mut vertices: Option<Vec<Vec3>> = None;
        let mut texture_coords: Option<Vec<Vec2>> = None;
        let mut normals: Option<Vec<Vec3>> = None;
        let mut faces: Vec<Face> = Vec::new();
        let mut minimum_point: Option<Vec3> = None;
        let mut maximum_point: Option<Vec3> = None;

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(positions) = reader.read_positions() 
            {
                let mut min: [f32; 3] = [0.0, 0.0, 0.0];
                let mut max: [f32; 3] = [0.0, 0.0, 0.0];
                vertices = Some(positions.map(|vertex| {
                    for i in 0..3 {
                        min[i] = min[i].min(vertex[i]);
                        max[i] = max[i].max(vertex[i]);
                    }
                    Vec3::new(vertex[0], vertex[1], vertex[2])
                }).collect());

                minimum_point = Some(Vec3::from_array(min));
                maximum_point = Some(Vec3::from_array(max));
            }

            if let Some(normals_iter) = reader.read_normals() 
            {
                normals = Some(normals_iter.map(|normal| Vec3::new(normal[0], normal[1], normal[2])).collect());
            }

            if let Some(texture_coords_iter) = reader.read_tex_coords(0) 
            {
                texture_coords = Some(texture_coords_iter.into_f32().map(|texture_coord| Vec2::new(texture_coord[0], texture_coord[1])).collect());
            }

            if let Some(indices_iter) = reader.read_indices() {
                let mut iterator = indices_iter.into_u32().into_iter();
                while let Some(first_index) = iterator.next() {
                    let second_index = iterator.next().unwrap_or_else(|| {
                        println!("Error processing mesh! Not enough indcies!");
                        0
                    });
                    let third_index = iterator.next().unwrap_or_else(|| {
                        println!("Error processing mesh! Not enough indcies!");
                        0
                    });
                    faces.push(Face {
                        indices: [first_index, second_index, third_index]
                    });
                }
            }
        }

        Self {
            faces,
            vertices: vertices.unwrap_or(Vec::new()),
            texture_coords: texture_coords.unwrap_or(Vec::new()),
            normals,
            axis_aligned_bounding_box: AxisAlignedBoundingBox::new_from_points(minimum_point.unwrap_or(Vec3::splat(0.0)), maximum_point.unwrap_or(Vec3::splat(0.0)))
        }
    }

    pub(crate) fn intersect(&self, local_ray: &Ray) -> IntersectionInfo {
        let mut does_intersect = false;
        let mut intersection_parameter = f32::INFINITY;
        let mut face_index = None;

        let (does_aabb_intersect, _) = self.axis_aligned_bounding_box.intersect_ray(local_ray);
        
        if does_aabb_intersect {
            for (index, face) in self.faces.iter().enumerate() {
                let vertex1 = self.vertices[face.indices[0] as usize];
                let vertex2 = self.vertices[face.indices[1] as usize];
                let vertex3 = self.vertices[face.indices[2] as usize];
                let intersection_info = Triangle::intersect_triangle(&local_ray, &vertex1, &vertex2, &vertex3);
                if intersection_info.does_intersect && intersection_info.intersection_parameter < intersection_parameter {
                    does_intersect = true;
                    intersection_parameter = intersection_info.intersection_parameter;
                    face_index = Some(FaceIndex {
                        mesh_index: 0,
                        face_index: index as u32
                    });
                }
            }
        }

        IntersectionInfo { does_intersect, intersection_parameter, mesh_info: face_index }
    }

    pub(crate) fn get_normals_of_face(&self, face_index: u32) -> Vec3 {
        let indices = self.faces[face_index as usize].indices;
        match &self.normals  {
            Some(normals_vec) => {
                ((normals_vec[indices[0] as usize] + normals_vec[indices[1] as usize] + normals_vec[indices[2] as usize]) / 3.0).normalize()
            },
            None => {
                let vertex1 = self.vertices[indices[0] as usize];
                let vertex2 = self.vertices[indices[1] as usize];
                let vertex3 = self.vertices[indices[2] as usize];
                (vertex2 - vertex1).cross(vertex3 - vertex1)
            }
        }
    }
}
