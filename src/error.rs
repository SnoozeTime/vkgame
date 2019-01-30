use std::error::Error;
use std::convert::From;
use std::fmt;


// vulkano errors...
use vulkano::memory::DeviceMemoryAllocError;
use vulkano::swapchain::{CapabilitiesError, SwapchainCreationError};
use image::ImageError;
use vulkano::image::sys::ImageCreationError;
use vulkano::sampler::SamplerCreationError;
use vulkano::sync::FlushError;
use vulkano::command_buffer::DrawIndexedError;

pub type TwResult<T> = Result<T, TwError>;

#[derive(Debug)]
pub enum TwError {
    // Mine
    ModelLoading(String),
    RenderingSystemInitialization(String),

    // Vulkano
    VkDeviceMemoryAlloc(DeviceMemoryAllocError),
    VkCapabilities(CapabilitiesError),
    VkSwapchainCreation(SwapchainCreationError),

    // Image and texture
    ImageLoading(ImageError),
    VkImageCreation(ImageCreationError),
    VkSamplerCreation(SamplerCreationError),

    // GpuFuture error
    VkFutureFlush(FlushError),


    // Drawing
    VkDrawIndexed(DrawIndexedError),

    // Saving/recovering scene.
    Io(std::io::Error),
    JsonSerde(serde_json::Error),
}

impl fmt::Display for TwError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TwError::ModelLoading(ref x) => write!(f, "{}", x),
            TwError::RenderingSystemInitialization(ref x) => write!(f, "{}", x),
            TwError::VkDeviceMemoryAlloc(ref x) => write!(f, "{}", x),
            TwError::VkCapabilities(ref x) => write!(f, "{}", x),
            TwError::VkSwapchainCreation(ref x) => write!(f, "{}", x),
            TwError::ImageLoading(ref x) => write!(f, "{}", x),
            TwError::VkImageCreation(ref x) => write!(f, "{}", x),
            TwError::VkSamplerCreation(ref x) => write!(f, "{}", x),
            TwError::VkFutureFlush(ref x) => write!(f, "{}", x),
            TwError::VkDrawIndexed(ref x) => write!(f, "{}", x),
            TwError::Io(ref x) => write!(f, "{}", x),
            TwError::JsonSerde(ref x) => write!(f, "{}", x),
        }
    }
}


impl Error for TwError {
    fn description(&self) -> &str {
        match *self {
            TwError::ModelLoading(ref x) => x,  
            TwError::RenderingSystemInitialization(ref x) => x,  
            TwError::VkDeviceMemoryAlloc(ref x) => x.description(),  
            TwError::VkCapabilities(ref x) => x.description(),  
            TwError::VkSwapchainCreation(ref x) => x.description(),  
            TwError::ImageLoading(ref x) => x.description(),  
            TwError::VkImageCreation(ref x) => x.description(),  
            TwError::VkSamplerCreation(ref x) => x.description(),  
            TwError::VkFutureFlush(ref x) => x.description(),  
            TwError::VkDrawIndexed(ref x) => x.description(),  
            TwError::Io(ref x) => x.description(),  
            TwError::JsonSerde(ref x) => x.description(),  
        }
    }
}

impl From<DeviceMemoryAllocError> for TwError {
    fn from(err: DeviceMemoryAllocError) -> Self {
        TwError::VkDeviceMemoryAlloc(err)
    }
}

impl From<CapabilitiesError> for TwError {
    fn from(err: CapabilitiesError) -> Self {
        TwError::VkCapabilities(err)
    }
}

impl From<SwapchainCreationError> for TwError {
    fn from(err: SwapchainCreationError) -> Self {
        TwError::VkSwapchainCreation(err)
    }
}

impl From<ImageError> for TwError {
    fn from(err: ImageError) -> Self {
        TwError::ImageLoading(err)
    }
}

impl From<ImageCreationError> for TwError {
    fn from(err: ImageCreationError) -> Self {
        TwError::VkImageCreation(err)
    }
}

impl From<SamplerCreationError> for TwError {
    fn from(err: SamplerCreationError) -> Self {
        TwError::VkSamplerCreation(err)
    }
}

impl From<FlushError> for TwError {
    fn from(err: FlushError) -> Self {
        TwError::VkFutureFlush(err)
    }
}

impl From<DrawIndexedError> for TwError {
    fn from(err: DrawIndexedError) -> Self {
        TwError::VkDrawIndexed(err)
    }
}


impl From<std::io::Error> for TwError {
    fn from(err: std::io::Error) -> Self {
        TwError::Io(err)
    }
}


impl From<serde_json::Error> for TwError {
    fn from(err: serde_json::Error) -> Self {
        TwError::JsonSerde(err)
    }
}


