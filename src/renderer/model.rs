use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use std::path::Path;
use vulkano::device::Device;
use std::sync::Arc;
use std::collections::HashMap;
use tobj;
use crate::error::{TwError, TwResult};
/*
 *  The vertex data that will be passed as input to
 *  the graphic pipeline.
 * */
#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    position: [f32; 3],
    texcoords: [f32; 2],
}

impl Vertex {
    fn new(x: f32, y: f32, z: f32, tx: f32, ty: f32) -> Self {
        let position = [x, y, z];
        let texcoords = [tx, ty];
        Vertex {
            position,
            texcoords,
        }
    }
}
vulkano::impl_vertex!(Vertex, position, texcoords);

/*
 * Model that are loaded in GPU memory
 * */
#[derive(Debug)]
pub struct Model {
    pub vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pub index_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
}

impl Model {

    // Uses the tinyobj library to load mesh from obj file.
    pub fn load_from_obj(device: Arc<Device>, filepath: &Path) -> TwResult<Model> {

        // TODO just support one mesh
        let box_obj = tobj::load_obj(filepath);
        let (mut models, _materials) = box_obj.unwrap();
        let mut model = models.pop().unwrap();

        let mesh = &mut model.mesh;
        let mut indices = Vec::new();
        indices.append(&mut mesh.indices);

        // Verify everything is consistent
        if mesh.positions.len() % 3 != 0 {
            return Err(TwError::ModelLoading("Mesh position vector length is not a multiple of 3.".to_owned()));
        }
        if mesh.texcoords.len() % 2 != 0 {
            return Err(TwError::ModelLoading("Mesh texture vector length is not a multiple of 2.".to_owned()));
        }
        if (mesh.positions.len() / 3) != (mesh.texcoords.len() /2) {
            return Err(TwError::ModelLoading(
                    format!("Number of positions ({}) does not correspond to number of texture coords ({})",
                    mesh.positions.len() / 3,
                    mesh.texcoords.len() / 2)));
        }

        let mut vertices = Vec::new();
        for v in 0..mesh.positions.len() / 3 {
            vertices.push(Vertex::new(mesh.positions[3 * v],
                                      mesh.positions[3 * v + 1],
                                      mesh.positions[3 * v + 2],
                                      mesh.texcoords[2 * v],
                                      1.0 - mesh.texcoords[2 * v + 1]));
        }

        Self::load_from_vec(device, vertices, indices) 
    }

    pub fn load_from_vec(device: Arc<Device>, vertices: Vec<Vertex>, indices: Vec<u32>) -> TwResult<Model> {

        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            vertices.iter().cloned())?;

        let index_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            indices.iter().cloned())?;

        Ok(Model {
            vertex_buffer,
            index_buffer,
        })
    }

}

// Just store the models so that they can be referenced by name in the scene.
pub struct ModelManager {
    pub models: HashMap<String, Model>,
}

impl ModelManager {

    pub fn new() -> Self {
        ModelManager {
            models: HashMap::new(),
        }
    }

    pub fn load_model(
        &mut self,
        model_name: String,
        filename: &Path,
        device: Arc<Device>) -> TwResult<()> {

        let model = Model::load_from_obj(device, filename)?;

        self.models.insert(model_name, model);
        
        Ok(())
    }
                      
}

