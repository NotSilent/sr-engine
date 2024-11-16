use ash::{vk, Device};

//pub struct CommandBufferHelpers;

pub fn single_image_pipeline_barrier(
    device: &Device,
    command_buffer: vk::CommandBuffer,
    src_stage_mask: vk::PipelineStageFlags,
    dst_stage_mask: vk::PipelineStageFlags,
    src_access_mask: vk::AccessFlags,
    dst_access_mask: vk::AccessFlags,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    image: vk::Image,
    aspect_mask: vk::ImageAspectFlags,
) {
    let subresource_range = vk::ImageSubresourceRange::default()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let image_memory_barriers = [vk::ImageMemoryBarrier::default()
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask)
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(subresource_range)];

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            src_stage_mask,
            dst_stage_mask,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &image_memory_barriers,
        )
    };
}

// impl CommandBufferHelpers {
//     pub fn single_image_pipeline_barrier(
//         device: &Device,
//         command_buffer: vk::CommandBuffer,
//         src_stage_mask: vk::PipelineStageFlags,
//         dst_stage_mask: vk::PipelineStageFlags,
//         src_access_mask: vk::AccessFlags,
//         dst_access_mask: vk::AccessFlags,
//         old_layout: vk::ImageLayout,
//         new_layout: vk::ImageLayout,
//         image: vk::Image,
//         aspect_mask: vk::ImageAspectFlags,
//     ) {
//         let subresource_range = vk::ImageSubresourceRange::default()
//             .aspect_mask(aspect_mask)
//             .base_mip_level(0)
//             .level_count(1)
//             .base_array_layer(0)
//             .layer_count(1);

//         let image_memory_barriers = [vk::ImageMemoryBarrier::default()
//             .src_access_mask(src_access_mask)
//             .dst_access_mask(dst_access_mask)
//             .old_layout(old_layout)
//             .new_layout(new_layout)
//             .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
//             .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
//             .image(image)
//             .subresource_range(subresource_range)];

//         unsafe {
//             device.cmd_pipeline_barrier(
//                 command_buffer,
//                 src_stage_mask,
//                 dst_stage_mask,
//                 vk::DependencyFlags::empty(),
//                 &[],
//                 &[],
//                 &image_memory_barriers,
//             )
//         };
//     }
// }
