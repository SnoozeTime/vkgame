/// This is where all the imgui-rs related functions are.
use vulkano::pipeline::viewport::Viewport;
use vulkano::device::Queue;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::framebuffer::{Subpass, RenderPassAbstract};
use imgui::{ImGui, Ui, ImDrawVert};
use vulkano::impl_vertex;
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet};
use vulkano::sampler::{Sampler, SamplerAddressMode, Filter, MipmapMode};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::swapchain::Surface;
use std::error::Error;
use vulkano::image::ImmutableImage;
use vulkano::format::R8G8B8A8Unorm;
use vulkano::sync::GpuFuture;
use std::sync::Arc;

use winit::Window;

pub struct Texture {
    pub texture: Arc<ImmutableImage<R8G8B8A8Unorm>>,
    pub sampler: Arc<Sampler>,
}


/// Vertex for UI only has 2d data. The texture is actually for the fonts
#[derive(Debug, Clone)]
struct UiVertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}
impl_vertex!(UiVertex,
             position, uv, color);

impl From<ImDrawVert> for UiVertex {
    fn from(v: ImDrawVert) -> Self {
        UiVertex {
            position: [v.pos.x, v.pos.y],
            uv: [v.uv.x, v.uv.y],
            color: convert_color(v.col),
        }
    }
}

fn convert_color(col: u32) -> [f32;4] {
    [(col & 0xFF) as f32 /255.0,
    ((col >> 8) & 0xFF) as f32 /255.0, 
    ((col >> 16) & 0xFF) as f32 /255.0, 
    ((col >> 24) & 0xFF) as f32 /255.0]
}


/// Passed to the renderer during the render pass.
struct DrawData {
    state: DynamicState,
    vtx_buf: Arc<CpuAccessibleBuffer<[UiVertex]>>,
    idx_buf: Arc<CpuAccessibleBuffer<[u32]>>, 
}

/// Mega structure for the UI! :) 
/// UI will have its own pipeline (shaders and stuff) and its own texture for fonts. 
/// At first, we create vertex and index buffer on the go, but ideally we would
/// store everything in one buffer.
pub struct GuiRenderer {
    queue: Arc<Queue>,

    // The UI state
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    vs: ui_vs::Shader,
    fs: ui_fs::Shader,

    font_texture: Texture,
    //dimensions: [u32; 2],
    current_viewport: Viewport,
}

impl GuiRenderer {


    pub fn new<R>(imgui: &mut ImGui,
                  surface: Arc<Surface<Window>>,
                  subpass: Subpass<R>, 
                  queue: Arc<Queue>) -> Self 
        where R: RenderPassAbstract + Send + Sync + 'static {

            let device = queue.device().clone();
            let window = surface.window();
            // Load the font texture
            // --------------------------------------------
            let (texture, texture_future) = imgui.prepare_texture(|handle| {
                let r = vulkano::image::immutable::ImmutableImage::from_iter(
                    handle.pixels.iter().cloned(),
                    vulkano::image::Dimensions::Dim2d { width: handle.width, height: handle.height },
                    vulkano::format::R8G8B8A8Unorm,
                    queue.clone()).unwrap();

                r
            });

            let sampler = Sampler::new(
                device.clone(),
                Filter::Linear,
                Filter::Linear,
                MipmapMode::Linear,
                SamplerAddressMode::ClampToEdge,
                SamplerAddressMode::ClampToEdge,
                SamplerAddressMode::ClampToEdge,
                0.0, 1.0, 0.0, 0.0).unwrap();

            texture_future
                .then_signal_fence_and_flush().unwrap()
                .wait(None).unwrap(); 

            let font_texture = Texture {
                texture,
                sampler,
            };

            // Window size and so on
            // -----------------------
            let dimensions = if let Some(dimensions) = window.get_inner_size() {
                // convert to physical pixels
                let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                // The window no longer exists so exit the application.
                panic!("Wwwwwttttfffff");
            };
            let current_viewport = Viewport {
                origin: [0.0, 0.0],
                dimensions: [dimensions[0] as f32, dimensions[1] as f32],
                depth_range: 0.0 .. 1.0,
            };

            // The graphic pipeline
            // -----------------------------------------------
            let vs = ui_vs::Shader::load(device.clone()).unwrap();
            let fs = ui_fs::Shader::load(device.clone()).unwrap();
            let pipeline = GuiRenderer::build_pipeline(
                queue.clone(),
                subpass,
                &vs, &fs);

            GuiRenderer {
                queue,
                font_texture,
                pipeline,
                vs,
                fs,
                current_viewport,
            }
        }

    fn build_pipeline<R>(queue: Arc<Queue>,
                         subpass: Subpass<R>,
                         vs: &ui_vs::Shader,
                         fs: &ui_fs::Shader) -> Arc<GraphicsPipelineAbstract + Send + Sync>
        where R: RenderPassAbstract + Send + Sync + 'static {

            Arc::new(
                GraphicsPipeline::start()
                .vertex_input_single_buffer::<UiVertex>()
                .vertex_shader(vs.main_entry_point(), ())
                .triangle_list()
                .front_face_clockwise()
                .blend_alpha_blending() // necessary for font transparency
                .viewports_scissors_dynamic(1)
                .fragment_shader(fs.main_entry_point(), ())
                .render_pass(subpass)
                .build(queue.device().clone())
                .unwrap()
            )
        }

    /// Render the given ui!
    pub fn render<'a>(&mut self,
                      mut builder: AutoCommandBufferBuilder, ui: Ui<'a>) -> AutoCommandBufferBuilder {
        let mut to_draw = Vec::new();
        let (width, height) = ui.imgui().display_size();
        let push_constants = ui_vs::ty::PushConstants {
            scale: [2.0 / width as f32, 2.0 / height as f32],
            translate: [-1.0, -1.0],
        }; 

        // TODO what to do with the result?
        let _render_result: Result<(), Box<Error>> = ui.render(|ui, mut draw_data| {

            draw_data.scale_clip_rects(ui.imgui().display_framebuffer_scale());
            for draw_list in &draw_data {
                //self.render_draw_list(surface, &draw_list, fb_size, matrix)?;
                let idx: Vec<u32> = draw_list.idx_buffer.iter()
                    .map(|index| u32::from(*index)).collect();

                let vtx: Vec<UiVertex> = draw_list.vtx_buffer.iter()
                    .map(|v| UiVertex::from(*v)).collect();

                // Create vertex and index buffers here.
                let vertex_buffer = CpuAccessibleBuffer::from_iter(
                    self.queue.device().clone(),
                    BufferUsage::all(),
                    vtx.iter().cloned()).unwrap();
                let index_buffer = CpuAccessibleBuffer::from_iter(
                    self.queue.device().clone(),
                    BufferUsage::all(),
                    idx.iter().cloned()
                ).unwrap();

                //let mut idx_start: usize = 0;
                for cmd in draw_list.cmd_buffer {

                    let state = DynamicState {
                        line_width: None,
                        viewports: Some(vec![self.current_viewport.clone()]),
                        scissors: Some(vec![vulkano::pipeline::viewport::Scissor {
                            origin: [std::cmp::max(cmd.clip_rect.x as i32, 0), std::cmp::max(cmd.clip_rect.y as i32, 0)],
                            dimensions: [(cmd.clip_rect.z - cmd.clip_rect.x) as u32, (cmd.clip_rect.w - cmd.clip_rect.y) as u32],
                        }]),
                    };
                    to_draw.push(DrawData {
                        idx_buf: index_buffer.clone(),
                        vtx_buf: vertex_buffer.clone(),
                        state,
                    });
                }
            }

            Ok(())
        });

        let tex_set = Arc::new(
            PersistentDescriptorSet::start(self.pipeline.clone(), 0)
            .add_sampled_image(
                self.font_texture.texture.clone(),
                self.font_texture.sampler.clone()).unwrap()
            .build().unwrap()
        );


        for draw_data in to_draw {
            builder = builder
                .draw_indexed(
                    self.pipeline.clone(),
                    &draw_data.state,
                    vec![draw_data.vtx_buf.clone()],
                    draw_data.idx_buf.clone(),
                    tex_set.clone(),
                    push_constants)
                .unwrap();
        }

        builder
    }


    pub fn rebuild_pipeline<R>(&mut self,
                            subpass: Subpass<R>)
        where R: RenderPassAbstract + Send + Sync + 'static {
        self.pipeline = GuiRenderer::build_pipeline(self.queue.clone(),
            subpass, &self.vs, &self.fs);
    }
}

pub mod ui_vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "shaders/gui.vert"
    }
}

pub mod ui_fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "shaders/gui.frag"
    }
}


