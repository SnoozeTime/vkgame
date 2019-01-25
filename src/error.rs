use std::error::Error;
use std::convert::From;
use std::fmt;


// vulkano errors...
use vulkano::memory::DeviceMemoryAllocError;
use vulkano::swapchain::{CapabilitiesError, SwapchainCreationError};

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
}

impl fmt::Display for TwError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TwError::ModelLoading(ref x) => write!(f, "{}", x),
            TwError::RenderingSystemInitialization(ref x) => write!(f, "{}", x),
            TwError::VkDeviceMemoryAlloc(ref x) => write!(f, "{}", x),
            TwError::VkCapabilities(ref x) => write!(f, "{}", x),
            TwError::VkSwapchainCreation(ref x) => write!(f, "{}", x),
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

