use std::collections::HashMap;

use ash::{vk, Device};
use gpu_allocator::vulkan::Allocator;

use crate::buffer::{Buffer, VulkanResource};

pub struct BufferManager {
    device: Device,
    command_pool: vk::CommandPool,
    buffers: HashMap<String, Buffer>,
}

impl BufferManager {
    pub fn new(device: &Device, command_pool: vk::CommandPool) -> Self {
        Self {
            device: device.clone(),
            command_pool,
            buffers: HashMap::new(),
        }
    }

    pub fn destroy(&mut self, allocator: &mut Allocator) {
        for buffer in self.buffers.values_mut() {
            buffer.release(&self.device, allocator);
        }
        self.buffers.clear();
    }

    pub fn add_buffer<T>(
        &mut self,
        name: &str,
        allocator: &mut Allocator,
        queue: vk::Queue,
        data: &[T],
        usage: vk::BufferUsageFlags,
    ) {
        if self.buffers.contains_key(name) {
            return;
        }

        self.buffers.insert(
            name.to_string(),
            Buffer::new(
                &self.device,
                allocator,
                queue,
                self.command_pool,
                data,
                name,
                usage,
            ),
        );
    }

    pub fn get_buffer(&self, name: &str) -> &Buffer {
        self.buffers.get(name).unwrap()
    }
}
