use xenofrost::core::math::{Vec2, Vec3};

struct Face {
    indices: [u32; 3],
}

pub(crate) struct Mesh {
    faces: Vec<Face>,
    vertices: Vec<Vec3>,
    texture_coords: Vec<Vec2>,
    normals: Option<Vec<Vec3>>,
}

impl Mesh {
    pub(crate) fn create_mesh(mesh: &gltf::Mesh, buffers: &Vec<gltf::buffer::Data>) -> Self {
        let mut vertices: Option<Vec<Vec3>> = None;
        let mut texture_coords: Option<Vec<Vec2>> = None;
        let mut normals: Option<Vec<Vec3>> = None;
        let mut faces: Vec<Face> = Vec::new();

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            if let Some(positions) = reader.read_positions() 
            {
                vertices = Some(positions.map(|vertex| Vec3::new(vertex[0], vertex[1], vertex[2])).collect());
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
        }
    }
}
