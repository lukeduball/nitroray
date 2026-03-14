use gltf::Gltf;

use crate::mesh::Mesh;

struct Model {
    meshes: Vec<Mesh>
}

impl Model {
    fn load_model(model_name: &str) -> Self {
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
}

#[cfg(test)]
mod model_loading_test {
    use crate::model::Model;

    #[test]
    fn load_model_test() {
        let model = Model::load_model("res/models/plane.gltf");
        let mesh = &model.meshes[0];
        assert_eq!(model.meshes.len(), 1);
    }
}