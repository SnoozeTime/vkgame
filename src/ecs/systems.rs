use winit::{WindowBuilder, Window};
use vulkano::command_buffer::{DrawIndexedError, DynamicState, AutoCommandBufferBuilder};
use vulkano::descriptor::descriptor_set::{PersistentDescriptorSet};
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use vulkano_win::VkSurfaceBuild;
use vulkano_win;

use cgmath::Matrix4;
use std::sync::Arc;

use crate::renderer::{Renderer, vs};

use super::components::TransformComponent;
use super::ECS;

pub struct RenderingSystem<'a> {
    surface: Arc<Surface<Window>>,
    renderer: Renderer<'a>,    
}

impl<'a> RenderingSystem<'a> {

    fn new(instance: &'a Arc<Instance>, events_loop: &mut winit::EventsLoop) -> Self {
        // Get the surface and window. Window is from winit library
        let surface = WindowBuilder::new()
            .build_vk_surface(&events_loop, instance.clone())
            .expect("Cannot create vk_surface");

        // TODO error handling
        let mut renderer = Renderer::new(&instance, surface.clone()).unwrap();

        Self::init_textures(&mut renderer);
        Self::init_models(&mut renderer);

        RenderingSystem {
            surface,
            renderer,
        }
    }

    fn init_textures(render_system: &mut Renderer) {
        render_system.load_texture("bonjour".to_string(),
        std::path::Path::new("src/image_img.png"),
        93, 93).unwrap();
        render_system.load_texture("white".to_string(),
        std::path::Path::new("src/white.png"),
        93, 93).unwrap();
    }

    fn init_models(render_system: &mut Renderer) {
        render_system.load_model("cube".to_string(), std::path::Path::new("cube.obj")).expect("Cannot load model");

    }


    pub fn render(&mut self, ecs: &ECS) {

        if let Some((mut buffer, next_image_info)) = self.renderer.start_render() {

            let (view, proj) = ecs.camera.get_vp(); 

            // Naive rendering right now. Do not order or anything.
            for (idx, model_component) in ecs.model_components
                .iter()
                    .enumerate()
                    .filter(|(_, x)| x.is_some())
                    .map(|(i, x)| (i, x.as_ref().unwrap())){


                        if let Some(Some(ref transform)) = ecs.transform_components.get(idx) {
                            let texture = self.renderer.texture_manager.textures.get(
                                &model_component.texture_name
                            ).unwrap();

                            // BUILD DESCRIPTOR SETS.
                            //
                            // 1. For texture
                            let tex_set = Arc::new(
                                PersistentDescriptorSet::start(self.renderer.pipeline.pipeline.clone(), 1)
                                .add_sampled_image(texture.texture.clone(), texture.sampler.clone()).unwrap()
                                .build().unwrap()
                            );


                            let model = self.renderer.model_manager.models.get(
                                &model_component.mesh_name
                            ).unwrap();

                            let uniform_buffer_subbuffer = {
                                let uniform_data = create_mvp(&transform, &view, &proj);
                                self.renderer.uniform_buffer.next(uniform_data).unwrap()
                            };

                            let set = Arc::new(PersistentDescriptorSet::start(self.renderer.pipeline.pipeline.clone(), 0)
                                               .add_buffer(uniform_buffer_subbuffer).unwrap()
                                               .build().unwrap()
                            );


                            buffer =  buffer.draw_indexed(self.renderer.pipeline.pipeline.clone(),
                            &DynamicState::none(),
                            vec![model.vertex_buffer.clone()],
                            model.index_buffer.clone(),
                            (set.clone(), tex_set.clone()),
                            ()).unwrap();


                        }
                    }

            self.renderer.finish_render(buffer, next_image_info);
        }

    }
}

fn create_mvp(t: &TransformComponent, view: &Matrix4<f32>, proj: &Matrix4<f32>) -> vs::ty::Data {
    let scale = t.scale;
    let model = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z)
        * Matrix4::from_translation(t.position);


    vs::ty::Data {
        model: model.into(),
        view: (*view).into(),
        proj: (*proj).into(),
    }


}


