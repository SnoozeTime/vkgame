use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::Queue,
    memory::DeviceMemoryAllocError,
};

use std::sync::Arc;
// just collection of util functions that are used across the renderer files
//

/// A vertex that represent minimal information for a point in 2D
#[derive(Debug, Clone)]
pub struct Vertex2d {
    /// position of the vertex (x, y)
    position: [f32; 2],

    /// Texture coordinates
    uv: [f32; 2],
}
vulkano::impl_vertex!(Vertex2d, position, uv);

/// Create a vertex buffer than contains data for drawing a quad
/// also returns the indexes
pub fn create_quad(
    queue: Arc<Queue>,
) -> Result<
    (
        Arc<CpuAccessibleBuffer<[Vertex2d]>>,
        Arc<CpuAccessibleBuffer<[u32]>>,
    ),
    DeviceMemoryAllocError,
> {
    let buf = CpuAccessibleBuffer::from_iter(
        queue.device().clone(),
        BufferUsage::all(),
        [
            Vertex2d {
                position: [-1.0, -1.0],
                uv: [0.0, 0.0],
            },
            Vertex2d {
                position: [-1.0, 1.0],
                uv: [0.0, 1.0],
            },
            Vertex2d {
                position: [1.0, -1.0],
                uv: [1.0, 0.0],
            },
            Vertex2d {
                position: [1.0, 1.0],
                uv: [1.0, 1.0],
            },
        ]
        .iter()
        .cloned(),
    )?;

    let indexes = CpuAccessibleBuffer::from_iter(
        queue.device().clone(),
        BufferUsage::all(),
        [0, 1, 2, 2, 1, 3].iter().cloned(),
    )?;

    Ok((buf, indexes))
}
