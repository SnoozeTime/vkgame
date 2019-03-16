use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;

use vulkano::device::{Device, DeviceExtensions, Features};

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};


use vulkano::command_buffer::{CommandBuffer, AutoCommandBufferBuilder};
use vulkano::sync::GpuFuture;

fn main() {

    let instance = Instance::new(None, &InstanceExtensions::none(), None)
        .expect("Failed to create instance");

    let physical = PhysicalDevice::enumerate(&instance).next().expect("No device available");

    let queue_family = physical.queue_families()
        .find(|&q| q.supports_graphics())
        .expect("Could not find a graphical queue family");

    let (device, mut queues) = {
        Device::new(physical, &Features::none(), &DeviceExtensions::none(),
                    [(queue_family, 0.5)].iter().cloned())
            .expect("Failed to create device")
    };

    let queue = queues.next().unwrap();

    // Creating a buffer
    // ------------------
    let source_content = 0 .. 64;
    let source = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        source_content).expect("Failed to create buffer");


    let dest_content = (0..64).map(|_| 0);
    let dest = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        dest_content).expect("Failed to create buffer");

    // Command buffer to do operation on GPU
    // ----------------
    let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family())
        .unwrap()
        .copy_buffer(source.clone(), dest.clone()).unwrap()
        .build().unwrap();

    // submission and synchronization.
   let finished = command_buffer.execute(queue.clone()).unwrap(); 

   // Wait for the operation to be over.
   finished.then_signal_fence_and_flush().unwrap().wait(None).unwrap();

   let src_content = source.read().unwrap();
   let dest_content = dest.read().unwrap();

   dbg!(&*src_content);
   dbg!(&*dest_content);
}
