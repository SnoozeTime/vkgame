use vulkano::command_buffer::AutoCommandBuffer;
use crate::camera::Camera;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::DynamicState;
use cgmath::SquareMatrix;
use cgmath::{Matrix4, Vector3, Rad};
use vulkano::device::Queue;
use vulkano::pipeline::{
    GraphicsPipelineAbstract, GraphicsPipeline,
    viewport::Viewport,
};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::buffer::BufferUsage;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;

use crate::renderer::model::{Vertex, Model};
use crate::ecs::components::TransformComponent;

use std::sync::Arc;
use std::path::Path;
use std::iter;

pub struct SkyboxSystem {
    queue: Arc<Queue>,
    dimensions: [u32; 2],

    // pipeline + shaders
    vs: vs::Shader,
    fs: fs::Shader,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    uniform_buffer: CpuBufferPool<vs::ty::Data>,

    // Data for the cube.
    cube: Model,
    skybox_color: fs::ty::PushConstants,
    //skybox_img: Arc<ImmutableImage<vulkano::format::Format>>,
    //skybox_sampler: Arc<Sampler>,
}

impl SkyboxSystem {

    pub fn new<R>(queue: Arc<Queue>,
                  subpass: Subpass<R>,
                  color: [f32; 3]) -> Self 
        where R: RenderPassAbstract + Clone + Send + Sync + 'static
        {

            let cube = Model::load_from_obj(queue.device().clone(),
            Path::new("assets/cube.obj")).unwrap();

            // cubemap are textures with 6 images.
            // left, right, bottom, top, back, and front
            //let img_negx = image::load_from_memory_with_format(include_bytes!("../../assets/skyblue.png"), ImageFormat::PNG).unwrap().to_rgba();
//            let img_posx = image::load_from_memory_with_format(include_bytes!("../../assets/skyblue.png"), ImageFormat::PNG).unwrap().to_rgba();
//            let img_posy = image::load_from_memory_with_format(include_bytes!("../../assets/skyblue.png"), ImageFormat::PNG).unwrap().to_rgba();
//            let img_negy = image::load_from_memory_with_format(include_bytes!("../../assets/skyblue.png"), ImageFormat::PNG).unwrap().to_rgba();
//            let img_negz = image::load_from_memory_with_format(include_bytes!("../../assets/skyblue.png"), ImageFormat::PNG).unwrap().to_rgba();
//            let img_posz = image::load_from_memory_with_format(include_bytes!("../../assets/skyblue.png"), ImageFormat::PNG).unwrap().to_rgba();


//            let cubemap_images = [img_posx, img_negx, img_posy, img_negy, img_posz, img_negz];
//            let mut image_data: Vec<u8> = Vec::new();
//
//            for image in cubemap_images.into_iter() {
//                let mut image0 = image.clone().into_raw().clone();
//                image_data.append(&mut image0);
//            }
//
//            let (skybox_img, gpu_future) = {
//                ImmutableImage::from_iter(image_data.iter().cloned(),
//                Dimensions::Cubemap { size: 512},
//                Format::R8G8B8A8Srgb,
//                queue.clone()).unwrap()
//            };
//
//            let skybox_sampler = Sampler::new(
//                queue.device().clone(),
//                Filter::Linear,
//                Filter::Linear,
//                MipmapMode::Nearest,
//                SamplerAddressMode::Repeat,
//                SamplerAddressMode::Repeat,
//                SamplerAddressMode::Repeat, 0.0, 1.0, 0.0, 0.0).unwrap();
//
//            // TODO need a better way
//            gpu_future
//                .then_signal_fence_and_flush().unwrap()
//                .wait(None).unwrap(); 


            // -----------------------------
            let vs = vs::Shader::load(queue.device().clone()).unwrap();
            let fs = fs::Shader::load(queue.device().clone()).unwrap();
            let pipeline = SkyboxSystem::build_pipeline(queue.clone(), subpass, [1, 1], &vs, &fs);

            let skybox_color = fs::ty::PushConstants {

                color: [color[0]/255.0,
                        color[1]/255.0,
                        color[2]/255.0,
                        1.0],
                    };

            let dimensions = [1, 1];
            let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(queue.device().clone(), BufferUsage::all());
            SkyboxSystem {
                uniform_buffer,
                dimensions,
                queue,
                cube,
                //skybox_img,
                //skybox_sampler,
                skybox_color,
                vs,
                fs,
                pipeline,
            }
        }

    fn get_proj(&self) -> Matrix4<f32> {
        let aspect = self.dimensions[0] as f32 / self.dimensions[1] as f32;
        let proj = cgmath::perspective(Rad(std::f32::consts::FRAC_PI_4),
        aspect,
        0.1,
        256.0);
        let mut the_fix = Matrix4::identity();
        the_fix[1][1] = -1.0;
        the_fix[2][3] = 0.5;
        the_fix[2][2] = 0.5;

        the_fix * proj
    }

    pub fn draw(&self,
                camera: &mut Camera) -> AutoCommandBuffer {

        let (mut view, _) = camera.get_vp(); 
        // need to remove the rotation from the view matrix.
        view[0][3] = 0.0;
        view[1][3] = 0.0;
        view[2][3] = 0.0;
        view[0][3] = 0.0;
        view[1][3] = 0.0;
        view[2][3] = 0.0;
        view[3][3] = 0.0;

        let proj = self.get_proj();


        let transform = TransformComponent {
            position: Vector3::new(0.0, 0.0, 0.0),
            rotation: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(2000000.0, 2000000.0, 2000000.0),
        };
        let uniform_data = {
            let data = create_mvp(&transform, &view, &proj);
            self.uniform_buffer.next(data).unwrap()
        };

        let set = Arc::new(PersistentDescriptorSet::start(self.pipeline.clone(), 0)
                           .add_buffer(uniform_data).unwrap()
                           .build().unwrap());

//        let tex_set = Arc::new(
//            PersistentDescriptorSet::start(self.pipeline.clone(), 1)
//            .add_sampled_image(self.skybox_img.clone(), self.skybox_sampler.clone()).unwrap().build().unwrap());
//
        let mut builder = AutoCommandBufferBuilder::secondary_graphics(
            self.queue.device().clone(),
            self.queue.family(),
            self.pipeline.clone().subpass()).unwrap();

        // Only one cube to draw.
        builder = builder.draw_indexed(self.pipeline.clone(),
        &DynamicState::none(),
        vec![self.cube.vertex_buffer.clone()],
        self.cube.index_buffer.clone(),
        set,
        self.skybox_color.clone()).unwrap();

        builder.build().unwrap()
    }

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
            .depth_stencil_simple_depth()
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
            self.dimensions = dimensions;
            self.pipeline = SkyboxSystem::build_pipeline(
                self.queue.clone(),
                subpass, dimensions, &self.vs, &self.fs
            );
        }

    pub fn recompile_shaders(&mut self) {
        println!("Recompiling");
        if let Ok(_) = self.fs.recompile(self.queue.device().clone()) {
            self.rebuild_pipeline(self.pipeline.clone().subpass(), self.dimensions);
        }
    }
}

pub fn create_mvp(t: &TransformComponent, view: &Matrix4<f32>, proj: &Matrix4<f32>) -> vs::ty::Data {
    let scale = t.scale;
    let model = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
        * Matrix4::from_translation(t.position);


    vs::ty::Data {
        model: model.into(),
        view: (*view).into(),
        proj: (*proj).into(),
    }
}

mod vs {
    
    twgraph_shader::twshader! {

        path: "assets/shaders/skybox.vert",
        kind: "vertex",
        input: [
            {
                format: R32G32B32Sfloat,
                name: "position"
            },
            {
                format: R32G32Sfloat,
                name: "texcoords"
            },
            {
                format: R32G32B32Sfloat,
                name: "normals"
            }
        ],
        output: [
            {
                format: R32G32B32Sfloat,
                name: "frag_tex_coords"
            }
        ],
        descriptors: [
            {
                name: Data,
                ty: Buffer,
                binding: 0,
                set: 0,
                data: [
                    (model, "mat4"),
                    (view, "mat4"),
                    (proj, "mat4")
                ]
            }
        ],
    }
}

mod fs {
    
    twgraph_shader::twshader! {

        path: "assets/shaders/skybox_color.frag",
        kind: "fragment",
        input: [
            // vec3 frag_tex_coords
            {
                format: R32G32B32Sfloat,
                name: "frag_tex_coords"
            }
        ],
        output: [
            // vec4 f_color
            {
                format: R32G32B32A32Sfloat,
                name: "f_color"
            }
        ],
        // vec4 of 4 bytes :)
        push_constants: {
            name: PushConstants,
            ranges: [(color, 4)],
        }
    }
}
