use ash::{vk, Device};
use gpu_allocator::{
    vulkan::{Allocation, AllocationCreateDesc, AllocationScheme, Allocator},
    MemoryLocation,
};

pub struct ImageCreateInfo {
    pub extent: vk::Extent3D,
    pub image_type: vk::ImageType,
    pub format: vk::Format,
    pub usage: vk::ImageUsageFlags,
    pub view_type: vk::ImageViewType,
    pub aspect_mask: vk::ImageAspectFlags,
}

pub struct Image {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    allocation: Option<Allocation>,
}

impl Image {
    pub fn new(device: &Device, allocator: &mut Allocator, create_info: &ImageCreateInfo) -> Self {
        let image_create_info = vk::ImageCreateInfo::default()
            .image_type(create_info.image_type)
            .format(create_info.format)
            .extent(create_info.extent)
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(create_info.usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(&[])
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = unsafe { device.create_image(&image_create_info, None).unwrap() };
        let requirements = unsafe { device.get_image_memory_requirements(image) };

        let description = AllocationCreateDesc {
            name: "image",
            // TODO: name: &format!("{}_staging", name),
            requirements,
            location: MemoryLocation::GpuOnly,
            linear: false,
            // TODO: allocation_scheme: AllocationScheme::DedicatedImage(image),
            allocation_scheme: AllocationScheme::GpuAllocatorManaged,
        };

        let allocation = allocator.allocate(&description).unwrap();

        // TODO: ?
        unsafe {
            device
                .bind_image_memory(image, allocation.memory(), allocation.offset())
                .unwrap()
        };

        let view_create_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(create_info.view_type)
            .format(create_info.format)
            .components(
                vk::ComponentMapping::default()
                    .r(vk::ComponentSwizzle::R)
                    .g(vk::ComponentSwizzle::G)
                    .b(vk::ComponentSwizzle::B)
                    .a(vk::ComponentSwizzle::A),
            )
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(create_info.aspect_mask)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        let image_view = unsafe { device.create_image_view(&view_create_info, None).unwrap() };

        Self {
            image,
            image_view,
            allocation: Some(allocation),
        }
    }

    pub fn destroy(&mut self, device: &Device, allocator: &mut Allocator) {
        allocator.free(self.allocation.take().unwrap());

        unsafe {
            device.destroy_image_view(self.image_view, None);
            device.destroy_image(self.image, None);
        }
    }
}
