use std::path::PathBuf;
use std::sync::Arc;
use vulkano::device::{Device, Queue};
use vulkano::format::Format;
use vulkano::image::{Dimensions, ImmutableImage};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::sync::GpuFuture;

use std::collections::HashMap;

use crate::error::TwResult;

pub struct Texture {
    pub texture: Arc<ImmutableImage<vulkano::format::Format>>,
    pub sampler: Arc<Sampler>,
    pub width: u32,
    pub height: u32,
}

impl Texture {
    /*
     * Load a texture from file. Will return the texture and the GpuFuture
     * that will tell when the texture is loaded in the GPU memory. You need
     * to wait for that future otherwise vulkano will panic with buffer non
     * initialized.
     * */
    pub fn load(
        filename: PathBuf,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> TwResult<(Texture, Box<GpuFuture>)> {
        let ((texture, tex_future), width, height) = {
            let image = image::open(filename)?.to_rgba();
            let width = image.width();
            let height = image.height();
            let image_data = image.into_raw().clone();

            (
                ImmutableImage::from_iter(
                    image_data.iter().cloned(),
                    Dimensions::Dim2d { width, height },
                    Format::R8G8B8A8Srgb,
                    queue.clone(),
                )?,
                width,
                height,
            )
        };

        let sampler = Sampler::new(
            device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0,
            1.0,
            0.0,
            0.0,
        )?;

        Ok((
            Texture {
                texture,
                sampler,
                width,
                height,
            },
            Box::new(tex_future),
        ))
    }
}

// -----------------------------------------
// Keep all the textures at the same place
// -----------------------------------------
//
pub struct TextureManager {
    pub textures: HashMap<String, Texture>,
}

impl TextureManager {
    pub fn new() -> TextureManager {
        TextureManager {
            textures: HashMap::new(),
        }
    }

    // Load texture and directly wait for it to be in the Gpu.
    pub fn load_texture(
        &mut self,
        texture_name: String,
        filename: PathBuf,
        device: Arc<Device>,
        queue: Arc<Queue>,
    ) -> TwResult<()> {
        let (texture, gpu_future) = Texture::load(filename, device.clone(), queue.clone())?;

        gpu_future.then_signal_fence_and_flush()?.wait(None)?;

        self.textures.insert(texture_name, texture);
        Ok(())
    }
}
