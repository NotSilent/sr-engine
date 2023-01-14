use crate::buffer::{Buffer, VulkanResource};
use crate::vertex::Vertex;
use ash::{vk, Device};
use gpu_allocator::vulkan::Allocator;

pub struct Mesh {
    pub index_buffer: Buffer,
    pub vertex_buffer: Buffer,
}

impl Mesh {
    pub fn new(
        device: &Device,
        allocator: &mut Allocator,
        queue: vk::Queue,
        command_pool: vk::CommandPool,
        name: &str,
        vertices: Vec<Vertex>,
        indices: Vec<u16>,
    ) -> Self {
        let index_buffer = Buffer::new(
            device,
            allocator,
            queue,
            command_pool,
            &indices,
            &format!("{}_index_buffer", name),
            vk::BufferUsageFlags::INDEX_BUFFER,
        );

        let vertex_buffer = Buffer::new(
            device,
            allocator,
            queue,
            command_pool,
            &vertices,
            &format!("{}_vertex_buffer", name),
            vk::BufferUsageFlags::VERTEX_BUFFER,
        );

        Mesh {
            index_buffer,
            vertex_buffer,
        }
    }

    pub fn get_index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    pub fn get_vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }
}

impl VulkanResource for Mesh {
    fn release(&mut self, device: &Device, allocator: &mut Allocator) {
        self.index_buffer.release(device, allocator);
        self.vertex_buffer.release(device, allocator);
    }
}
