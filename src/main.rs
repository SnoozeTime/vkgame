use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass, RenderPassAbstract};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError};
use vulkano::swapchain;
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::sync;

use vulkano_win::VkSurfaceBuild;

use winit::{KeyboardInput, VirtualKeyCode, EventsLoop, Window, WindowBuilder, Event, WindowEvent};

use std::sync::Arc;

struct Sprite {
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    index_buffer: Arc<CpuAccessibleBuffer<[u16]>>,
}

impl Sprite {

    fn new(device: Arc<Device>, vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>, indices: Vec<u16>) -> Sprite {

        let index_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(), 
            BufferUsage::all(), 
            indices.iter().cloned()).unwrap();

        Sprite {
            vertex_buffer,
            index_buffer 
        }
    }
}

#[derive(Debug, Clone)]
struct Vertex { 
    position: [f32; 2],
    color: [f32; 4]
}
vulkano::impl_vertex!(Vertex, position, color);


fn main() {

    // this is an Arc to instance. (non-mut dynamic ref)
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("Could not create instance")
    };

    // List of device.
    // In real life -> Maybe all the device cannot draw. Should leave the device
    // choice to the user... Here we get the first one that comes
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("Cannot get PhysicalDevice");


    // Get the surface and window. Window is from winit library
    let mut events_loop = EventsLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .expect("Cannot create vk_surface");
    let window = surface.window();

    // Get the queue to write command to the GPU. queues should support graphics
    // and should be able to write to our surface
    let queue_family = physical.queue_families()
        .find(|&q| {
            q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
        }).expect("Cannot find queue family");

    // Now we can initialize the vulkan device. Needs five parameters
    // - which physical device to connect to
    // - a list of optional features and extensions that our program needs to work
    // correctly on. Here we only need khr_swapchain extension that allows us to draw
    // to a window
    // - List of layers to enable (?) very niche so use None in example.
    // - list of queues that we are going to use. Exact parameters is a iteraotr
    // which items are (Queue, f32) where float is priority between 0 and 1. (hint)
    let device_ext = DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() };
    let (device, mut queues) = Device::new(physical,
                                           physical.supported_features(),
                                           &device_ext,
                                           [(queue_family, 0.5)].iter().cloned()).expect("Could not create device");

    let queue = queues.next().unwrap();
    // Create the swapchain. Creating a swapchain allocates the color buffers that
    // will contain the image that will be visible on screen.
    let (mut swapchain, images) = {

        let caps = surface.capabilities(physical).unwrap();
        println!("{:?}", caps);
        let usage = caps.supported_usage_flags;
        // alpha mode indicates how the alpha value of the final image will behave.
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
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
        Swapchain::new(device.clone(),
        surface.clone(),
        caps.min_image_count,
        format,
        initial_dimensions,
        1,
        usage,
        &queue,
        SurfaceTransform::Identity,
        alpha,
        PresentMode::Fifo,
        true,
        None).unwrap()

    };


    // Now let's create a buffer that will store the shape of our triangle.
    let vertex_buffer = {
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), [
                                       Vertex { position: [0.5, 0.5],
                                       color: [0.0, 1.0, 0.0, 0.5],},
                                       Vertex { position: [0.5, 0.25],
                                       color: [0.2, 1.0, 0.1, 1.0],},
                                       Vertex { position: [0.0, 0.5],
                                       color: [0.3, 0.9, 0.9, 1.0],},
                                       Vertex { position: [-0.25, -0.1],
                                       color: [0.0, 1.0, 0.0, 1.0],},
                                       Vertex { position: [-0.5, -0.25],
                                       color: [1.0, 0.4, 0.0, 1.0],},
                                       Vertex { position: [0.0, -0.5],
                                       color: [0.0, 1.0, 0.1, 1.0],},
        ].iter().cloned()).unwrap()
    };

    let triangle1 = Sprite::new(device.clone(), vertex_buffer.clone(), vec![0, 1, 2]);
    let triangle2 = Sprite::new(device.clone(), vertex_buffer.clone(), vec![3, 4, 5]);
    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();

    let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(device.clone(), BufferUsage::all());

    // at this point opengl init would be finished but vulkna requires more.
    // Render pass is an object that describes where the output of the graphic
    // pipeline will go. It describes the layout of the images where the color
    // depth and/or stencil info will be written.
    let render_pass = Arc::new(vulkano::single_pass_renderpass!(
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

    // Define the pipeline
    let pipeline = Arc::new(GraphicsPipeline::start()
                            // need to indicate the layout of vertices.
                            .vertex_input_single_buffer()
                            .vertex_shader(vs.main_entry_point(), ())
                            .triangle_list()
                            .viewports_dynamic_scissors_irrelevant(1)
                            .fragment_shader(fs.main_entry_point(), ())
                            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
                            .build(device.clone())
                            .unwrap());
    // Dynamic viewports allow us to recreate just the viewport when the window is resized
    // Otherwise we would have to recreate the whole pipeline.
    let mut dynamic_state = DynamicState { line_width: None, viewports: None, scissors: None };

    // The render pass we created above only describes the layout of our framebuffers. Before we
    // can draw we also need to create the actual framebuffers.
    // Since we need to draw to multiple images, we are going to create a different framebuffer for
    // each image.
    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);


    // Initialization is finally finished!
    let mut recreate_swapchain = false;
    // In the loop below we are going to submit commands to the GPU. Submitting a command produces
    // an object that implements the `GpuFuture` trait, which holds the resources for as long as
    // they are in use by the GPU.
    //
    // Destroying the `GpuFuture` blocks until the GPU is finished executing it. In order to avoid
    // that, we store the submission of the previous frame here.
    let mut previous_frame_end = Box::new(sync::now(device.clone())) as Box<GpuFuture>;

    let mut offset_x = 0.0;
    let mut offset_y = 0.0;
    let mut offset_z = 0.0;
    loop {
        previous_frame_end.cleanup_finished();

        if recreate_swapchain {
            let dimensions = if let Some(dimensions) = window.get_inner_size() {
                let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                return;
            };

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                Err(err) => panic!("{:?}", err)
            };

            swapchain = new_swapchain;
            // Because framebuffers contains an Arc on the old swapchain, we need to
            // recreate framebuffers as well.
            framebuffers = window_size_dependent_setup(&new_images, render_pass.clone(), &mut dynamic_state);

            recreate_swapchain = false;
        }

        let uniform_buffer_subbuffer = {
            let uniform_data = vs::ty::Data {
                offset: [offset_x, offset_y, offset_z],
            };

            uniform_buffer.next(uniform_data).unwrap()
        };

        let set = Arc::new(
            PersistentDescriptorSet::start(pipeline.clone(), 0)
            .add_buffer(uniform_buffer_subbuffer).unwrap()
            .build().unwrap());

        // Before we can draw on the output, we have to *acquire* an image from the
        // swapchain. If no image is available, (which happens if you submit draw
        // commands too quickly), then the function will block.
        // This operation returns the index of the image that we are allowed to draw upon.
        // None can be a timeout instead.
        let (image_num, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                recreate_swapchain = true;
                continue;
            },
            Err(err) => panic!("{:?}", err)
        };

        // Specify the color to clear the framebuffer with.
        let clear_values = vec!([0.0, 0.0, 1.0, 1.0].into());

        // In order to draw, we have to build a *command buffer*. The command buffer object holds
        // the list of commands that are going to be executed.
        //
        // Building a command buffer is an expensive operation (usually a few hundred
        // microseconds), but it is known to be a hot path in the driver and is expected to be
        // optimized.
        //
        // Note that we have to pass a queue family when we create the command buffer. The command
        // buffer will only be executable on that given queue family.
        let indices: [u16; 3] = [0, 1, 2];
        let mut command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
            // Before we can draw, we have to *enter a render pass*. There are two methods to do
            // this: `draw_inline` and `draw_secondary`. The latter is a bit more advanced and is
            // not covered here.
            //
            // The third parameter builds the list of values to clear the attachments with. The API
            // is similar to the list of attachments when building the framebuffers, except that
            // only the attachments that use `load: Clear` appear in the list.
            .begin_render_pass(framebuffers[image_num].clone(), false, clear_values)
            .unwrap()

            // We are now inside the first subpass of the render pass. We add a draw command.
            // The last two parameters contain the list of resources to pass to the shaders.
            // Since we used an `EmptyPipeline` object, the objects have to be `()`.
            .draw_indexed(pipeline.clone(),
            &dynamic_state,
            triangle1.vertex_buffer.clone(),
            triangle1.index_buffer.clone(),
            set.clone(),
            ())
            .unwrap();

        command_buffer = command_buffer.draw_indexed(pipeline.clone(),
        &dynamic_state,
        triangle2.vertex_buffer.clone(),
        triangle2.index_buffer.clone(),
        set.clone(),
        ())
            .unwrap()

            // We leave the render pass by calling `draw_end`. Note that if we had multiple
            // subpasses we could have called `next_inline` (or `next_secondary`) to jump to the
            // next subpass.
            .end_render_pass()
            .unwrap();

        // Finish building the command buffer by calling `build`.
        let command_buffer = command_buffer.build().unwrap();


        let future = previous_frame_end.join(acquire_future)
            .then_execute(queue.clone(), command_buffer).unwrap()

            // The color output is now expected to contain our triangle. But in order to show it on
            // the screen, we have to *present* the image by calling `present`.
            //
            // This function does not actually present the image immediately. Instead it submits a
            // present command at the end of the queue. This means that it will only be presented once
            // the GPU has finished executing the command buffer that draws the triangle.
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

        // Note that in more complex programs it is likely that one of `acquire_next_image`,
        // `command_buffer::submit`, or `present` will block for some time. This happens when the
        // GPU's queue is full and the driver has to wait until the GPU finished some work.
        //
        // Unfortunately the Vulkan API doesn't provide any way to not wait or to detect when a
        // wait would happen. Blocking may be the desired behavior, but if you don't want to
        // block you should spawn a separate thread dedicated to submissions.

        // Handling the window events in order to close the program when the user wants to close
        // it.
        let mut done = false;
        events_loop.poll_events(|ev| {
            if let Event::WindowEvent { event, ..} = ev {
                match event {
                    WindowEvent::CloseRequested => done = true,
                    WindowEvent::Resized(_) => recreate_swapchain = true,
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                ..
                            },
                            ..
                    } => {
                        match keycode {
                            VirtualKeyCode::Escape => done = true,
                            VirtualKeyCode::D => offset_x += 0.05,
                            VirtualKeyCode::W => offset_y -= 0.05,
                            VirtualKeyCode::A => offset_x -= 0.05,
                            VirtualKeyCode::S => offset_y += 0.05,
                            _ => (),
                        }
                    },
                    _ => (),
                }
            }});

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
            path: "shaders/triangle.vert"
        }
    }

    mod fs {
        vulkano_shaders::shader!{
            ty: "fragment",
            src: "
#version 450

layout(location = 0) out vec4 f_color;
layout(location = 0) in vec4 frag_color;

void main() {
    f_color = frag_color;
}
"
}
}


