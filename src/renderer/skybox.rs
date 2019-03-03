use vulkano::command_buffer::AutoCommandBuffer;
use vulkano::device::Queue;
use vulkano::pipeline::{
    GraphicsPipelineAbstract, GraphicsPipeline,
    viewport::Viewport,
};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};


use crate::renderer::model::{Vertex, Model};

use std::sync::Arc;
use std::path::Path;
use std::iter;

pub struct SkyboxSystem {
    queue: Arc<Queue>,

    // pipeline + shaders
    vs: vs::Shader,
    fs: fs::Shader,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,

    // Data for the cube.
    cube: Model,
}

impl SkyboxSystem {

    pub fn new<R>(queue: Arc<Queue>,
                  subpass: Subpass<R>) -> Self 
        where R: RenderPassAbstract + Clone + Send + Sync + 'static
    {

        let cube = Model::load_from_obj(queue.device().clone(),
                                        Path::new("assets/cube.obj")).unwrap();

        let vs = vs::Shader::load(queue.device().clone()).unwrap();
        let fs = fs::Shader::load(queue.device().clone()).unwrap();
        let pipeline = SkyboxSystem::build_pipeline(queue.clone(), subpass, [1, 1], &vs, &fs);

        SkyboxSystem {
            queue,
            cube,

            vs,
            fs,
            pipeline,
        }
    }

//    pub fn draw(&self) -> AutoCommandBuffer {
//
//    }

    fn build_pipeline<R>(
        queue: Arc<Queue>,
        subpass: Subpass<R>,
        dimensions: [u32; 2],
        vs: &vs::Shader,
        fs: &fs::Shader) -> Arc<GraphicsPipelineAbstract + Send + Sync> where R: RenderPassAbstract + Send + Sync + 'static {

        Arc::new(

            GraphicsPipeline::start()
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .viewports(iter::once(Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0 .. 0.99,
            }))
            .fragment_shader(fs.main_entry_point(), ())
            .render_pass(subpass)
            .build(queue.device().clone()).unwrap()

            )

    }

    pub fn rebuild_pipeline<R>(&mut self, subpass: Subpass<R>, dimensions: [u32; 2]) 
        where R: RenderPassAbstract + Clone + Send + Sync + 'static {
            self.pipeline = SkyboxSystem::build_pipeline(
                self.queue.clone(),
                subpass, dimensions, &self.vs, &self.fs
                );
        }
}


mod vs {

    vulkano_shaders::shader!{
        ty: "vertex",
        path: "shaders/skybox.vert"
    }
}

mod fs {

    vulkano_shaders::shader!{
        ty: "fragment",
        path: "shaders/skybox.frag"
    }
}
