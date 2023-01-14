use ash::{vk, Device};

pub struct CommandBuffer {}

impl CommandBuffer {
    pub fn pipeline_barrier(
        device: &Device,
        cmd: vk::CommandBuffer,
        src_stage_mask: vk::PipelineStageFlags,
        dst_stage_mask: vk::PipelineStageFlags,
        src_access_mask: vk::AccessFlags,
        dst_access_mask: vk::AccessFlags,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        src_queue_family_index: u32,
        dst_queue_family_index: u32,
        image: vk::Image,
        aspect_mask: vk::ImageAspectFlags,
    ) {
        unsafe {
            device.cmd_pipeline_barrier(
                cmd,
                src_stage_mask,
                dst_stage_mask,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[vk::ImageMemoryBarrier::builder()
                    .src_access_mask(src_access_mask)
                    .dst_access_mask(dst_access_mask)
                    .old_layout(old_layout)
                    .new_layout(new_layout)
                    .src_queue_family_index(src_queue_family_index)
                    .dst_queue_family_index(dst_queue_family_index)
                    .image(image)
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(aspect_mask)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1)
                            .build(),
                    )
                    .build()],
            )
        };
    }
}
