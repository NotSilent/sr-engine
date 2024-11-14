use ash::{vk, Device};
use gpu_allocator::vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator};
use gpu_allocator::MemoryLocation;
use std::intrinsics::copy_nonoverlapping;

pub struct Buffer {
    pub buffer_size: vk::DeviceSize,
    pub buffer: vk::Buffer,
    allocation: Option<Allocation>,
}

impl Buffer {
    pub fn new<T>(
        device: &Device,
        allocator: &mut Allocator,
        queue: vk::Queue,
        command_pool: vk::CommandPool,
        data: &[T],
        name: &str,
        usage: vk::BufferUsageFlags,
    ) -> Self {
        unsafe {
            let buffer_size = (data.len() * std::mem::size_of::<T>()) as vk::DeviceSize;

            let staging_buffer_create_info = vk::BufferCreateInfo::default()
                .size(buffer_size)
                .usage(usage | vk::BufferUsageFlags::TRANSFER_SRC);

            let staging_buffer = device
                .create_buffer(&staging_buffer_create_info, None)
                .unwrap();

            let requirements = device.get_buffer_memory_requirements(staging_buffer);

            let staging_buffer_allocation = allocator
                .allocate(&AllocationCreateDesc {
                    name: &format!("{}_staging", name),
                    requirements,
                    location: MemoryLocation::CpuToGpu,
                    linear: true, // Buffers are always linear
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                })
                .unwrap();

            device
                .bind_buffer_memory(
                    staging_buffer,
                    staging_buffer_allocation.memory(),
                    staging_buffer_allocation.offset(),
                )
                .unwrap();

            copy_nonoverlapping(
                data.as_ptr(),
                staging_buffer_allocation
                    .mapped_ptr()
                    .unwrap()
                    .cast()
                    .as_ptr(),
                data.len(),
            );

            let buffer_create_info = vk::BufferCreateInfo::default()
                .size(buffer_size)
                .usage(usage | vk::BufferUsageFlags::TRANSFER_DST);

            let buffer = device.create_buffer(&buffer_create_info, None).unwrap();

            let requirements = device.get_buffer_memory_requirements(buffer);

            let allocation = allocator
                .allocate(&AllocationCreateDesc {
                    name,
                    requirements,
                    location: MemoryLocation::GpuOnly,
                    linear: true, // Buffers are always linear
                    allocation_scheme: AllocationScheme::GpuAllocatorManaged,
                })
                .unwrap();

            device
                .bind_buffer_memory(buffer, allocation.memory(), allocation.offset())
                .unwrap();

            let allocate_info = vk::CommandBufferAllocateInfo::default()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1);

            let command_buffers = device.allocate_command_buffers(&allocate_info).unwrap();
            let cmd = command_buffers[0];

            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            device.begin_command_buffer(cmd, &begin_info).unwrap();
            device.cmd_copy_buffer(
                cmd,
                staging_buffer,
                buffer,
                &[vk::BufferCopy::default().size(buffer_size)],
            );
            device.end_command_buffer(cmd).unwrap();

            device
                .queue_submit(
                    queue,
                    &[vk::SubmitInfo::default().command_buffers(&command_buffers)],
                    vk::Fence::null(),
                )
                .unwrap();
            device.queue_wait_idle(queue).unwrap();

            device.free_command_buffers(command_pool, &command_buffers);

            allocator.free(staging_buffer_allocation).unwrap();
            device.destroy_buffer(staging_buffer, None);

            Self {
                buffer_size,
                buffer,
                allocation: Some(allocation),
            }
        }
    }
}

pub trait VulkanResource {
    fn release(&mut self, device: &Device, allocator: &mut Allocator);
}

impl VulkanResource for Buffer {
    fn release(&mut self, device: &Device, allocator: &mut Allocator) {
        allocator.free(self.allocation.take().unwrap()).unwrap();
        unsafe { device.destroy_buffer(self.buffer, None) };
    }
}
