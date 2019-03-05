// Frame is an abstraction of the work done during rendering.
// Instead of putting all the render pass logic in renderer/mod.rs, I will
// use this structure to provide a simpler API. 
// 
// The inital code is from the vulkano examples, modified to my needs
use std::sync::Arc;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::command_buffer::CommandBuffer;
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::framebuffer::{FramebufferAbstract, Framebuffer};
use vulkano::framebuffer::RenderPassAbstract;
use vulkano::framebuffer::Subpass;
use vulkano::image::AttachmentImage;
use vulkano::image::ImageAccess;
use vulkano::image::{ImageViewAccess};
use vulkano::image::ImageUsage;
use vulkano::sync::GpuFuture;
use vulkano::sampler::{Sampler, SamplerAddressMode, Filter, MipmapMode};
use vulkano::format::ClearValue;

use cgmath::{Vector3};

use super::point_lighting_system::PointLightingSystem;
use super::ambient_lighting_system::AmbientLightingSystem;
use super::directional_lighting_system::DirectionalLightingSystem;
use super::GBufferComponent;
use super::pp_system::PPSystem;
use super::skybox::SkyboxSystem;
// Renderpass description takes a lot of place so it is created here.
use crate::camera::Camera;
mod renderpass;

impl GBufferComponent {
    fn new(device: Arc<Device>,
           dimensions: [u32; 2], 
           format: Format, usage: ImageUsage) -> Self {

        let image = AttachmentImage::with_usage(
            device.clone(),
            dimensions, 
            format,
            usage).unwrap();

        let sampler = Sampler::new(
            device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Linear,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge,
            SamplerAddressMode::ClampToEdge, 0.0, 1.0, 0.0, 1.0).unwrap();

        GBufferComponent {
            image, sampler,
        }
    }
}

pub struct FrameSystem {
    // Queue used to render graphic
    queue: Arc<Queue>,

    // Will determine where are we drawing to.
    offscreen_render_pass: Arc<RenderPassAbstract + Send + Sync>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,

    // GBuffer
    diffuse_buffer: GBufferComponent,
    // Contains fragment_positions;
    frag_pos_buffer: GBufferComponent,
    // Contains the normals
    normals_buffer: GBufferComponent,
    // Depth buffer. will be recreated if needed.
    depth_buffer: GBufferComponent,

    // Lighting systems.
    point_lighting_system: PointLightingSystem,
    ambient_lighting_system: AmbientLightingSystem,
    directional_lighting_system: DirectionalLightingSystem,
    //pp_system: PPSystem,
    skybox_system: SkyboxSystem,
}

impl FrameSystem {

    pub fn new(queue: Arc<Queue>, final_output_format: Format) -> Self {


        let (offscreen_render_pass, render_pass) = renderpass::build_render_pass(
            queue.device().clone(), final_output_format);

        let usage = FrameSystem::get_image_usage();
        // most likely the dimensions are not good. It's ok, we'll recreate when creating
        // a new frame in case dimension does not match with final image.
        let depth_buffer = GBufferComponent::new(
            queue.device().clone(),
            [1, 1],
            Format::D16Unorm, usage);

        let frag_pos_buffer = GBufferComponent::new(
            queue.device().clone(),
            [1, 1],
            Format::R16G16B16A16Sfloat, usage);

        let normals_buffer = GBufferComponent::new(
            queue.device().clone(),
            [1, 1],
            Format::R16G16B16A16Sfloat, usage);

        let diffuse_buffer = GBufferComponent::new(
            queue.device().clone(),
            [1, 1],
            Format::A2B10G10R10UnormPack32, usage);


        let lighting_subpass = Subpass::from(offscreen_render_pass.clone(), 1).unwrap();
        let point_lighting_system = PointLightingSystem::new(
            queue.clone(),
            lighting_subpass.clone());
        let ambient_lighting_system = AmbientLightingSystem::new(
            queue.clone(),
            lighting_subpass.clone());
        let directional_lighting_system = DirectionalLightingSystem::new(
            queue.clone(),
            lighting_subpass.clone());

        //let pp_system = PPSystem::new(queue.clone(), lighting_subpass.clone()

        let skybox_subpass = Subpass::from(offscreen_render_pass.clone(), 2).unwrap();
        let skybox_system = SkyboxSystem::new(queue.clone(), skybox_subpass);

        FrameSystem {
            offscreen_render_pass,
            render_pass,
            queue: queue.clone(),
            depth_buffer,
            normals_buffer,
            diffuse_buffer,
            frag_pos_buffer,
            point_lighting_system,
            ambient_lighting_system,
            directional_lighting_system,
            skybox_system,
        }
    }

    #[inline]
    fn get_image_usage() -> ImageUsage {
        ImageUsage {
            sampled: true,
            input_attachment: true,
            .. ImageUsage::none()
        }
    }

    /// Return the subpass where we should write objects to the final image.
    #[inline]
    pub fn deferred_subpass(&self) -> Subpass<Arc<RenderPassAbstract + Send + Sync>> {
        Subpass::from(self.offscreen_render_pass.clone(), 0).unwrap()
    }

    #[inline]
    pub fn lighting_subpass(&self) -> Subpass<Arc<RenderPassAbstract + Send + Sync>> {
        Subpass::from(self.offscreen_render_pass.clone(), 1).unwrap()
    }

    #[inline]
    pub fn skybox_subpass(&self) -> Subpass<Arc<RenderPassAbstract + Send + Sync>> {
        Subpass::from(self.offscreen_render_pass.clone(), 2).unwrap()
    }

    /// Return the subpass where we should write the GUI to the final image
    #[inline]
    pub fn ui_subpass(&self) -> Subpass<Arc<RenderPassAbstract + Send + Sync>> {
        Subpass::from(self.render_pass.clone(), 0).unwrap()
    }


    fn rebuild_systems(&mut self,
                       dimensions: [u32; 2]) {
        self.point_lighting_system.rebuild_pipeline(
            self.lighting_subpass(),
            dimensions);
        self.ambient_lighting_system.rebuild_pipeline(
            self.lighting_subpass(),
            dimensions);
        self.skybox_system.rebuild_pipeline(self.skybox_subpass(), dimensions);
        self.directional_lighting_system.rebuild_pipeline(
            self.lighting_subpass(),
            dimensions);
    }

    /// Starts drawing a new frame. final image is the swapchain image that we are going
    /// to present.
    pub fn frame<F, I>(&mut self,
                       before_future: F,
                       final_image: I) -> Frame
        where F: GpuFuture + 'static,
              I: ImageAccess + ImageViewAccess + Clone + Send + Sync + 'static {

                  // First, reate attachments if dimension of image has changed.

                  let img_dims = ImageAccess::dimensions(&final_image).width_height();
                  if ImageAccess::dimensions(&self.depth_buffer.image).width_height() != img_dims {

                      let usage = FrameSystem::get_image_usage();
                      self.depth_buffer = GBufferComponent::new(
                          self.queue.device().clone(),
                          img_dims,
                          Format::D16Unorm, usage);

                      self.frag_pos_buffer = GBufferComponent::new(
                          self.queue.device().clone(),
                          img_dims,
                          Format::R16G16B16A16Sfloat, usage);

                      self.normals_buffer = GBufferComponent::new(
                          self.queue.device().clone(),
                          img_dims,
                          Format::R16G16B16A16Sfloat, usage);

                      self.diffuse_buffer = GBufferComponent::new(
                          self.queue.device().clone(),
                          img_dims,
                          Format::A2B10G10R10UnormPack32, usage);

                      self.rebuild_systems(img_dims);
                  }

                  let onscreen_framebuffer = Arc::new(Framebuffer::start(self.render_pass.clone()).add(final_image.clone()).unwrap().build().unwrap());

                  // Framebuffer contains all the attachments and output image.
                  let framebuffer = Arc::new(Framebuffer::start(self.offscreen_render_pass.clone())
                                             .add(final_image.clone()).unwrap()
                                             .add(self.diffuse_buffer.image.clone()).unwrap()
                                             .add(self.normals_buffer.image.clone()).unwrap()
                                             .add(self.frag_pos_buffer.image.clone()).unwrap()
                                             .add(self.depth_buffer.image.clone()).unwrap()
                                             .build().unwrap());

                  // Ok, begin the render pass now and return the Frame with all the information
                  let clear_values = vec!(
                      [0.0, 0.0, 0.0, 0.0].into(),
                      [0.0, 0.0, 0.0, 1.0].into(),
                      [0.0, 0.0, 0.0, 0.0].into(),
                      [0.0, 0.0, 0.0, 0.0].into(),
                      1f32.into());
                  let command_buffer = Some(AutoCommandBufferBuilder::primary_one_time_submit(
                          self.queue.device().clone(), self.queue.family()).unwrap()
                      .begin_render_pass(framebuffer.clone(), true, clear_values).unwrap());

                  Frame {
                      system: self,
                      onscreen_framebuffer,
                      before_main_cb_future: Some(Box::new(before_future)),
                      num_pass: 0,
                      command_buffer,
                  }

              }
}


/// Represent the current rendering.
pub struct Frame<'a> {
    system: &'a mut FrameSystem,

    onscreen_framebuffer: Arc<FramebufferAbstract + Send + Sync>,

    // 0 -> haven't begun yet
    // 1 -> finished drawing all the objects
    // 2 -> finished applying the lights.
    // 3 -> Finshed drawing the skybox
    // 4 -> finished drawing the GUI
    num_pass: u8,

    // wait before rendering
    before_main_cb_future: Option<Box<GpuFuture>>,

    command_buffer: Option<AutoCommandBufferBuilder>,
}


impl<'a> Frame<'a> {


    /// Order the different passes.
    pub fn next_pass<'f>(&'f mut self) -> Option<Pass<'f, 'a>> {

        match { let current_pass = self.num_pass; self.num_pass += 1; current_pass} {
            0 => {
                // Render pass has started but nothing is done yet.
                Some(Pass::Deferred(DrawPass { frame: self }))
            },
            1 => {
                self.command_buffer = Some(
                    self.command_buffer.take().unwrap()
                    .next_subpass(true).unwrap());
                Some(Pass::Lighting(LightingPass { frame: self }))
            },
            2 => {
                self.command_buffer = Some(
                    self.command_buffer.take().unwrap()
                    .next_subpass(true).unwrap());
                Some(Pass::Skybox(SkyboxPass { frame: self }))
            },
            3 => {
                // Finished drawing skybox, begin next
                // render pass
                let clear_values = vec!(ClearValue::None);

                let cmd_buf = self.command_buffer.take().unwrap()
                    .end_render_pass().unwrap()
                    .begin_render_pass(self.onscreen_framebuffer.clone(),
                    true,
                    clear_values).unwrap();

                self.command_buffer = Some(cmd_buf);

                Some(Pass::Gui(DrawPass { frame: self }))
            },
            4 => {
                // Finish render pass, schedule the command and return the future to wait
                // before rendering.
                let command_buffer = self.command_buffer.take().unwrap()
                    .end_render_pass().unwrap()
                    .build().unwrap();

                let after_main_cb = self.before_main_cb_future.take().unwrap()
                    .then_execute(self.system.queue.clone(), command_buffer)
                    .unwrap();
                Some(Pass::Finished(Box::new(after_main_cb)))
            },
            _ => None,
        }

    }

}


/// Allow to handle the pass differently by enum
/// Lifetimes are:
/// - 'f the frame.
/// - 's the FrameSystem
pub enum Pass<'f, 's: 'f>{
    Deferred(DrawPass<'f, 's>),
    Lighting(LightingPass<'f, 's>),
    Skybox(SkyboxPass<'f, 's>),
    Gui(DrawPass<'f, 's>),
    Finished(Box<GpuFuture>),
}


pub struct DrawPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> DrawPass<'f, 's> {

    pub fn execute<C>(&mut self, command_buffer: C)
        where C: CommandBuffer + Send + Sync + 'static {

            // sad.
            unsafe {
                self.frame.command_buffer = Some(
                    self.frame.command_buffer.take().unwrap()
                    .execute_commands(command_buffer).unwrap());
            }
        }
}

pub struct LightingPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> LightingPass<'f, 's> {

    pub fn ambient_light(&mut self, color: [f32; 3]) {

        let command_buffer = {
            self.frame.system.ambient_lighting_system.draw(
                self.frame.system.diffuse_buffer.image.clone(),
                color)

        };

        unsafe {
            self.frame.command_buffer = Some(
                self.frame.command_buffer.take().unwrap().execute_commands(command_buffer)
                .unwrap());
        }
    }

    pub fn point_light(&mut self, position: Vector3<f32>, color: [f32; 3]) {

        let command_buffer = {

            self.frame.system.point_lighting_system.draw(
                self.frame.system.diffuse_buffer.image.clone(),
                self.frame.system.normals_buffer.image.clone(),
                self.frame.system.frag_pos_buffer.image.clone(),
                self.frame.system.depth_buffer.image.clone(),
                position,
                color)
        };


        unsafe {
            self.frame.command_buffer = Some(
                self.frame.command_buffer.take().unwrap().execute_commands(command_buffer)
                .unwrap());

        }
    }

    pub fn directional_light(&mut self, direction: Vector3<f32>, color: [f32; 3]) {
 
        let command_buffer = {

            self.frame.system.directional_lighting_system.draw(
                self.frame.system.diffuse_buffer.image.clone(),
                self.frame.system.normals_buffer.image.clone(),
                self.frame.system.depth_buffer.image.clone(),
                direction,
                color)
        };


        unsafe {
            self.frame.command_buffer = Some(
                self.frame.command_buffer.take().unwrap().execute_commands(command_buffer)
                .unwrap());

        }       
    }
}

pub struct SkyboxPass<'f, 's: 'f> {
    frame: &'f mut Frame<'s>,
}

impl<'f, 's: 'f> SkyboxPass<'f, 's> {


    pub fn draw_skybox(&mut self,
                       camera: &mut Camera,) {

        let command_buffer = self.frame.system.skybox_system.draw(camera);

        unsafe {
            self.frame.command_buffer = Some(
                self.frame.command_buffer.take().unwrap().execute_commands(command_buffer).unwrap()
            );
        }
    }
}
