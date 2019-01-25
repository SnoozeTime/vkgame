use std::error::Error;
use std::convert::From;
use std::fmt;
use vulkano::memory::DeviceMemoryAllocError;

pub type TwResult<T> = Result<T, TwError>;

#[derive(Debug)]
pub enum TwError {
    ModelLoading(String),
    VkDeviceMemoryAlloc(DeviceMemoryAllocError),
}

impl fmt::Display for TwError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TwError::ModelLoading(ref x) => write!(f, "{}", x),
            TwError::VkDeviceMemoryAlloc(ref x) => write!(f, "{}", x),
        }
    }
}


impl Error for TwError {
    fn description(&self) -> &str {
        match *self {
            TwError::ModelLoading(ref x) => x,  
            TwError::VkDeviceMemoryAlloc(ref x) => x.description(),  
        }
    }
}

impl From<DeviceMemoryAllocError> for TwError {
    fn from(err: DeviceMemoryAllocError) -> Self {
        TwError::VkDeviceMemoryAlloc(err)
    }
}

