use std::path::Path;
use tobj;
use crate::error::{TwResult, TwError};
#[derive(Debug, Copy, Clone)]
struct Vertex {
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


#[derive(Debug)]
struct Mesh {
    vertices: Vec<Vertex>,
    // flattened list of indices. 
    indices: Vec<u32>,
}

fn create_mesh_from_tobj(mut model: tobj::Model) -> TwResult<Mesh> {
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
                                  mesh.texcoords[2 * v + 1]));
    }

    Ok(Mesh {
        vertices,
        indices,
    })
}


fn main() {
    let box_obj = tobj::load_obj(&Path::new("cube.obj"));
    let (mut models, materials) = box_obj.unwrap();
    println!("# of models:  {}", models.len());
    println!("# of materials:  {}", materials.len());


    let model = models.pop().unwrap();
    let mesh = create_mesh_from_tobj(model).unwrap();
    println!("{:?}", mesh);
}
