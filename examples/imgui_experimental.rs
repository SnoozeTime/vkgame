use imgui::{FontGlyphRange, ImFontConfig, ImGui, im_str, ImGuiCond, ImDrawVert};
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet};
use vulkano::sampler::{Sampler, SamplerAddressMode, Filter, MipmapMode};


use std::error::Error;
use imgui_winit_support;
use std::time::Instant;
use vulkano::image::ImmutableImage;
use vulkano::format::R8G8B8A8Unorm;

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
    color: [f32; 4],
}
impl_vertex!(Vertex,
             position, uv, color);

impl From<ImDrawVert> for Vertex {
    fn from(v: ImDrawVert) -> Self {
        Vertex {
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

pub struct Image {
    /// The actual image.
    pub image_access: Arc<ImmutableImage<R8G8B8A8Unorm>>,
    /// The width of the image.
    pub dimensions: vulkano::image::Dimensions,
}

struct DrawData {
    state: DynamicState,
    vtx_buf: Arc<CpuAccessibleBuffer<[Vertex]>>,
    idx_buf: Arc<CpuAccessibleBuffer<[u32]>>, 
}


//
// This is the only example that is entirely detailed. All the other examples avoid code
// duplication by using helper functions.
//
// This example assumes that you are already more or less familiar with graphics programming
// and that you want to learn Vulkan. This means that for example it won't go into details about
// what a vertex or a shader is.

// The `vulkano` crate is the main crate that you must use to use Vulkan.
#[macro_use]
extern crate vulkano;
// Provides the `shader!` macro that is used to generate code for using shaders.
extern crate vulkano_shaders;
// The Vulkan library doesn't provide any functionality to create and handle windows, as
// this would be out of scope. In order to open a window, we are going to use the `winit` crate.
extern crate winit;
// The `vulkano_win` crate is the link between `vulkano` and `winit`. Vulkano doesn't know about
// winit, and winit doesn't know about vulkano, so import a crate that will provide a link between
// the two.
extern crate vulkano_win;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass, RenderPassAbstract};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain};
use vulkano::swapchain;
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::sync;

use vulkano_win::VkSurfaceBuild;

use winit::{EventsLoop, Window, WindowBuilder, Event, WindowEvent};

use std::sync::Arc;

fn main() {
    // The first step of any Vulkan program is to create an instance.
    let instance = {
        let extensions = vulkano_win::required_extensions();

        // Now creating the instance.
        Instance::new(None, &extensions, None).unwrap()
    };
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
    // Some little debug infos.
    println!("Using device: {} (type: {:?})", physical.name(), physical.ty());


    let mut events_loop = EventsLoop::new();
    let surface = WindowBuilder::new()
        .with_dimensions((800, 600).into())
        .with_resizable(false)
        .build_vk_surface(&events_loop, instance.clone()).unwrap();
    let window = surface.window();
    let queue_family = physical.queue_families().find(|&q| {
        // We take the first queue that supports drawing to our window.
        q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
    }).unwrap();

    let device_ext = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
    let (device, mut queues) = Device::new(physical, physical.supported_features(), &device_ext,
                                           [(queue_family, 0.5)].iter().cloned()).unwrap();

    // Since we can request multiple queues, the `queues` variable is in fact an iterator. In this
    // example we use only one queue, so we just retrieve the first and only element of the
    // iterator and throw it away.
    let queue = queues.next().unwrap();

    // Before we can draw on the surface, we have to create what is called a swapchain. Creating
    // a swapchain allocates the color buffers that will contain the image that will ultimately
    // be visible on the screen. These images are returned alongside with the swapchain.
    let (swapchain, images) = {
        // Querying the capabilities of the surface. When we create the swapchain we can only
        // pass values that are allowed by the capabilities.
        let caps = surface.capabilities(physical).unwrap();

        let usage = caps.supported_usage_flags;

        // The alpha mode indicates how the alpha value of the final image will behave. For example
        // you can choose whether the window will be opaque or transparent.
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();

        // Choosing the internal format that the images will have.
        let format = caps.supported_formats[0].0;

        let initial_dimensions = if let Some(dimensions) = window.get_inner_size() {
            // convert to physical pixels
            let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
            [dimensions.0, dimensions.1]
        } else {
            // The window no longer exists so exit the application.
            return;
        };

        // Please take a look at the docs for the meaning of the parameters we didn't mention.
        Swapchain::new(device.clone(), surface.clone(), caps.min_image_count, format,
        initial_dimensions, 1, usage, &queue, SurfaceTransform::Identity, alpha,
        PresentMode::Fifo, true, None).unwrap()

    };


    // The next step is to create the shaders.
    //
    // The raw shader creation API provided by the vulkano library is unsafe, for various reasons.
    //
    // An overview of what the `vulkano_shaders::shader!` macro generates can be found in the
    // `vulkano-shaders` crate docs. You can view them at https://docs.rs/vulkano-shaders/
    //
    // TODO: explain this in details
    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();

    // At this point, OpenGL initialization would be finished. However in Vulkan it is not. OpenGL
    // implicitly does a lot of computation whenever you draw. In Vulkan, you have to do all this
    // manually.

    // The next step is to create a *render pass*, which is an object that describes where the
    // output of the graphics pipeline will go. It describes the layout of the images
    // where the colors, depth and/or stencil information will be written.
    let render_pass = Arc::new(single_pass_renderpass!(
            device.clone(),
            attachments: {
                // `color` is a custom name we give to the first and only attachment.
                color: {
                    // `load: Clear` means that we ask the GPU to clear the content of this
                    // attachment at the start of the drawing.
                    load: Clear,
                    // `store: Store` means that we ask the GPU to store the output of the draw
                    // in the actual image. We could also ask it to discard the result.
                    store: Store,
                    // `format: <ty>` indicates the type of the format of the image. This has to
                    // be one of the types of the `vulkano::format` module (or alternatively one
                    // of your structs that implements the `FormatDesc` trait). Here we use the
                    // same format as the swapchain.
                    format: swapchain.format(),
                    // TODO:
                    samples: 1,
                }
            },
            pass: {
                // We use the attachment named `color` as the one and only color attachment.
                color: [color],
                // No depth-stencil attachment is indicated with empty brackets.
                depth_stencil: {}
            }
    ).unwrap());

    // Before we draw we have to create what is called a pipeline. This is similar to an OpenGL
    // program, but much more specific.
    let pipeline = Arc::new(GraphicsPipeline::start()
                            // We need to indicate the layout of the vertices.
                            // The type `SingleBufferDefinition` actually contains a template parameter corresponding
                            // to the type of each vertex. But in this code it is automatically inferred.
                            .vertex_input_single_buffer()
                            // A Vulkan shader can in theory contain multiple entry points, so we have to specify
                            // which one. The `main` word of `main_entry_point` actually corresponds to the name of
                            // the entry point.
                            .vertex_shader(vs.main_entry_point(), ())
                            // The content of the vertex buffer describes a list of triangles.
                            .triangle_list()
                            .front_face_clockwise()
                            .blend_alpha_blending()
                            // Use a resizable viewport set to draw over the entire window
                            .viewports_scissors_dynamic(1)
                            // See `vertex_shader`.
                            .fragment_shader(fs.main_entry_point(), ())
                            // We have to indicate which subpass of which render pass this pipeline is going to be used
                            // in. The pipeline will only be usable from this particular subpass.
                            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                            // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
                            .build(device.clone())
                            .unwrap());

    // Dynamic viewports allow us to recreate just the viewport when the window is resized
    // Otherwise we would have to recreate the whole pipeline.
    let mut dynamic_state = DynamicState { line_width: None, viewports: None, scissors: None };

    // The render pass we created above only describes the layout of our framebuffers. Before we
    // can draw we also need to create the actual framebuffers.
    //
    // Since we need to draw to multiple images, we are going to create a different framebuffer for
    // each image.
    let framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

    // Initialization is finally finished!

    // In some situations, the swapchain will become invalid by itself. This includes for example
    // when the window is resized (as the images of the swapchain will no longer match the
    // window's) or, on Android, when the application went to the background and goes back to the
    // foreground.
    //
    // In this situation, acquiring a swapchain image or presenting it will return an error.
    // Rendering to an image of that swapchain will not produce any error, but may or may not work.
    // To continue rendering, we need to recreate the swapchain by creating a new swapchain.
    // Here, we remember that we need to do this for the next loop iteration.
    let mut recreate_swapchain = false;

    // In the loop below we are going to submit commands to the GPU. Submitting a command produces
    // an object that implements the `GpuFuture` trait, which holds the resources for as long as
    // they are in use by the GPU.
    //
    // Destroying the `GpuFuture` blocks until the GPU is finished executing it. In order to avoid
    // that, we store the submission of the previous frame here.
    let mut last_frame = Instant::now();

    // setup imgui
    let mut imgui = ImGui::init();
    let hidpi_factor = window.get_hidpi_factor();//.round();

    let font_size = (13.0 * hidpi_factor) as f32;

    imgui.fonts().add_default_font_with_config(
        ImFontConfig::new()
        .oversample_h(1)
        .pixel_snap_h(true)
        .size_pixels(font_size),
        );

    imgui.fonts().add_font_with_config(
        include_bytes!("mplus-1p-regular.ttf"),
        ImFontConfig::new()
        .merge_mode(true)
        .oversample_h(1)
        .pixel_snap_h(true)
        .size_pixels(font_size)
        .rasterizer_multiply(1.75),
        &FontGlyphRange::japanese(),
        );

    imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);


    imgui_winit_support::configure_keys(&mut imgui);

    let mut textures = Vec::new();

    let (texture, texture_future) = imgui.prepare_texture(|handle| {
        let r = vulkano::image::immutable::ImmutableImage::from_iter(
            handle.pixels.iter().cloned(),
            vulkano::image::Dimensions::Dim2d { width: handle.width, height: handle.height },
            vulkano::format::R8G8B8A8Unorm,
            queue.clone()).unwrap();

        textures.push(Image {
            image_access: r.0.clone(),
            dimensions: vulkano::image::Dimensions::Dim2d { width: handle.width, height: handle.height }
        });

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


    let tex_set = Arc::new(
        PersistentDescriptorSet::start(pipeline.clone(), 0)
        .add_sampled_image(texture.clone(), sampler.clone()).unwrap()
        .build().unwrap()
    );

    let mut previous_frame_end = Box::new(texture_future) as Box<GpuFuture>;
    let dimensions = if let Some(dimensions) = window.get_inner_size() {
        // convert to physical pixels
        let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
        [dimensions.0, dimensions.1]
    } else {
        // The window no longer exists so exit the application.
        return;
    };
     let current_viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0 .. 1.0,
    };

    loop {
        previous_frame_end.cleanup_finished();
        let (image_num, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                recreate_swapchain = true;
                continue;
            },
            Err(err) => panic!("{:?}", err)
        };


        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;
        imgui_winit_support::update_mouse_cursor(&imgui, &window);
        let frame_size = imgui_winit_support::get_frame_size(&window, hidpi_factor).unwrap();
        let ui = imgui.frame(frame_size, delta_s);
        ui.window(im_str!("Hello world"))
            .size((300.0, 100.0), ImGuiCond::FirstUseEver)
            .build(|| {
                ui.text(im_str!("Hello world!"));
                ui.text(im_str!("こんにちは世界！"));
                ui.text(im_str!("This...is...imgui-rs!"));
                ui.separator();
                let mouse_pos = ui.imgui().mouse_pos();
                ui.text(im_str!(
                        "Mouse Position: ({:.1},{:.1})",
                        mouse_pos.0,
                        mouse_pos.1
                ));

            });


        let mut to_draw = Vec::new();
            let (width, height) = ui.imgui().display_size();
    let push_constants = vs::ty::PushConstants {
        scale: [2.0 / width as f32, 2.0 / height as f32],
        translate: [-1.0, -1.0],
    }; 
    let _render_result: Result<(), Box<Error>> = ui.render(|ui, mut draw_data| {

            draw_data.scale_clip_rects(ui.imgui().display_framebuffer_scale());
            for draw_list in &draw_data {
                //self.render_draw_list(surface, &draw_list, fb_size, matrix)?;
                let idx: Vec<u32> = draw_list.idx_buffer.iter()
                    .map(|index| u32::from(*index)).collect();

                let vtx: Vec<Vertex> = draw_list.vtx_buffer.iter()
                    .map(|v| Vertex::from(*v)).collect();

                // Create vertex and index buffers here.
                let vertex_buffer = CpuAccessibleBuffer::from_iter(
                    device.clone(),
                    BufferUsage::all(),
                    vtx.iter().cloned()).unwrap();
                let index_buffer = CpuAccessibleBuffer::from_iter(
                    device.clone(),
                    BufferUsage::all(),
                    idx.iter().cloned()
                ).unwrap();

                for cmd in draw_list.cmd_buffer {

                    let state = DynamicState {
                        line_width: None,
                        viewports: Some(vec![current_viewport.clone()]),
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

        // Specify the color to clear the framebuffer with i.e. blue
        let clear_values = vec!([0.0, 0.0, 1.0, 1.0].into());

        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
            .begin_render_pass(framebuffers[image_num].clone(), false, clear_values)
            .unwrap()

            // We are now inside the first subpass of the render pass. We add a draw command.
            //
            // The last two parameters contain the list of resources to pass to the shaders.
            // Since we used an `EmptyPipeline` object, the objects have to be `()`.

            ;
        for draw_data in to_draw {
            command_buffer_builder = command_buffer_builder
                .draw_indexed(
                    pipeline.clone(),
                    &draw_data.state,
                    draw_data.vtx_buf.clone(),
                    draw_data.idx_buf.clone(),
                    tex_set.clone(),
                    push_constants)
                .unwrap();
        }
        // We leave the render pass by calling `draw_end`. Note that if we had multiple
        // subpasses we could have called `next_inline` (or `next_secondary`) to jump to the
        // next subpass.
        let command_buffer = command_buffer_builder.end_render_pass()
            .unwrap()

            // Finish building the command buffer by calling `build`.
            .build().unwrap();

        let future = previous_frame_end.join(acquire_future)
            .then_execute(queue.clone(), command_buffer).unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                previous_frame_end = Box::new(future) as Box<_>;
            }
            Err(FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame_end = Box::new(sync::now(device.clone())) as Box<_>;
            }
            Err(e) => {
                println!("{:?}", e);
                previous_frame_end = Box::new(sync::now(device.clone())) as Box<_>;
            }
        }

        let mut done = false;
        events_loop.poll_events(|ev| {

            //println!("WINDOW HIDPID {}, {}", window.get_hidpi_factor(), hidpi_factor);
            imgui_winit_support::handle_event(
                &mut imgui,
                &ev,
                window.get_hidpi_factor(),
                hidpi_factor,
                );
            match ev {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => done = true,
                Event::WindowEvent { event: WindowEvent::Resized(_), .. } => recreate_swapchain = true,
                _ => ()
            }
        });
        if done { return; }
    }
}

/// This method is called once during initialization, then again whenever the window is resized
fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState
) -> Vec<Arc<FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0 .. 1.0,
    };
    dynamic_state.viewports = Some(vec!(viewport));

    images.iter().map(|image| {
        Arc::new(
            Framebuffer::start(render_pass.clone())
            .add(image.clone()).unwrap()
            .build().unwrap()
        ) as Arc<FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>()
}
mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        path: "assets/shaders/gui.vert"
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        path: "assets/shaders/gui.frag" 
    }
}


