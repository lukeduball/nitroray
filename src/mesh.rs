use core::f32;
use std::rc::Rc;

use xenofrost::core::math::{Vec2, Vec3};

use crate::{data_structure::{Octree, OctreeNode, OctreeNodeType}, geometry::Triangle, math::AxisAlignedBoundingBox, object::{FaceIndex, IntersectionInfo}, ray::Ray};

const MIN_OBJECTS: u32 = 4;
const MAX_DEPTH: u32 = 5;

#[derive(Clone)]
struct Face {
    indices: [u32; 3],
}

pub(crate) struct Mesh {
    faces: Vec<Face>,
    vertices: Vec<Vec3>,
    texture_coords: Vec<Vec2>,
    normals: Option<Vec<Vec3>>,
    bounding_octree: Octree<u32>
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

        let vertices = vertices.unwrap_or(Vec::new());

        let mesh_aabb = AxisAlignedBoundingBox::new_from_points(minimum_point.unwrap_or(Vec3::splat(0.0)), maximum_point.unwrap_or(Vec3::splat(0.0)));
        let bounding_octree = Self::construct_octree(&faces, &vertices, mesh_aabb);

        Self {
            faces,
            vertices,
            texture_coords: texture_coords.unwrap_or(Vec::new()),
            normals,
            bounding_octree: bounding_octree
        }
    }

    fn construct_octree(faces: &Vec<Face>, vertices: &Vec<Vec3>, mesh_aabb: AxisAlignedBoundingBox) -> Octree<u32> {
        //The amount of faces is less than or equal to the minimum number of objects required to create a leaf in the octree so create a leaf node as the root node of the octree
        let root_node = if faces.len() as u32 <= MIN_OBJECTS {
            Rc::new(OctreeNode::new_leaf_node(mesh_aabb, (0..faces.len() as u32).collect()))
        } else {
            let root_branch_node = OctreeNode::new_branch_node(mesh_aabb);
            //Populate the children of the octree
            let rc_root_branch_node = Self::generate_octree_children(Rc::new(root_branch_node), &(0..faces.len() as u32).collect(), faces, vertices, 0);
            rc_root_branch_node
        };

        Octree::new(MIN_OBJECTS, MAX_DEPTH, root_node)
    }

    fn generate_octree_children(mut octree_node: Rc<OctreeNode<u32>>, faces_indices: &Vec<u32>, faces: &Vec<Face>, vertices: &Vec<Vec3>, depth: u32) -> Rc<OctreeNode<u32>> {
        let parent_center = octree_node.axis_aligned_bounding_box.get_center();
        let parent_half_distances = octree_node.axis_aligned_bounding_box.get_half_distances();

        let half_distances = parent_half_distances * 0.5;
        for children_index in 0..8_u32 {
            let mut overlapping_faces = Vec::new();

            // The total combinations of the three bits of numbers 0 to 7 represent the 8 different regions in the octree
            let x_sign = -1 + (children_index & 0b001) as i32 * 2;
            let y_sign = -1 + (children_index & 0b010) as i32 * 2;
            let z_sign = -1 + (children_index & 0b100) as i32 * 2;

            let center = parent_center + Vec3::new(parent_half_distances.x * x_sign as f32 * 0.5, parent_half_distances.y * y_sign as f32 * 0.5, parent_half_distances.z * z_sign as f32 * 0.5);
            let axis_aligned_bounding_box = AxisAlignedBoundingBox::new(center, half_distances);

            for face_index in faces_indices {
                let vertex1 = vertices[faces[*face_index as usize].indices[0] as usize];
                let vertex2 = vertices[faces[*face_index as usize].indices[1] as usize];
                let vertex3 = vertices[faces[*face_index as usize].indices[2] as usize];
                if axis_aligned_bounding_box.is_triangle_overlapping(&vertex1, &vertex2, &vertex3) {
                    overlapping_faces.push(*face_index);
                }
            }

            //If there are no overlapping faces, leave the node as None so it is not processed in the future
            if overlapping_faces.len() == 0 {
                continue;
            }

            //Otherwise, if the size is less than the min triangle count or the depth has reached the maximum depth stop creating nodes
            if overlapping_faces.len() as u32 <= MIN_OBJECTS || depth == MAX_DEPTH
            {
                let leaf_node = OctreeNode::new_leaf_node(axis_aligned_bounding_box, overlapping_faces.clone());
                let mutable_octree_node = Rc::get_mut(&mut octree_node).unwrap();
                if let OctreeNodeType::OctreeBranch { children } = &mut mutable_octree_node.octree_node_type {
                    children[children_index as usize] = Some(Rc::new(leaf_node));
                }
                continue;
            }
            //Otherwise, create a branch node and continue making the octree
            else
            {
                let branch_node = OctreeNode::new_branch_node(axis_aligned_bounding_box);
                let mutable_octree_node = Rc::get_mut(&mut octree_node).unwrap();
                let rc_branch_node = Self::generate_octree_children(Rc::new(branch_node), &overlapping_faces, faces, vertices, depth+1);
                if let OctreeNodeType::OctreeBranch { children } = &mut mutable_octree_node.octree_node_type {
                    children[children_index as usize] = Some(rc_branch_node);
                }
            }
        }

        octree_node
    }

    pub(crate) fn intersect(&self, local_ray: &Ray) -> IntersectionInfo {
        let mut does_intersect = false;
        let mut intersection_parameter = f32::INFINITY;
        let mut face_index = None;

        let (does_aabb_intersect, _) = self.bounding_octree.get_root_axis_aligned_bounding_box().intersect_ray(local_ray);
        
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
