use ash::{vk, Device};
use gpu_allocator::vulkan::Allocator;
use shadowmap_renderpass_consts::{DEPTH_CLEAR_VALUE, SHADOW_MAP_DIMENSIONS};

use crate::{
    command_buffer_helpers,
    draw_data::DrawData,
    image::{Image, ImageCreateInfo},
    pipeline_manager::{PipelineManager, ShadowMapMaterial},
    push_constants_data::PushConstantsData,
    render_pass_attachment_output::RenderPassAttachmentOutput,
};

// TODO: put under renderpasses and drop render_pass from name

pub mod deferred_renderpass_consts {
    use ash::vk;

    pub const COLOR: vk::Format = vk::Format::R8G8B8A8_UNORM;
    pub const NORMAL: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
    pub const POSITION: vk::Format = vk::Format::R16G16B16A16_SFLOAT;
    pub const DEPTH: vk::Format = vk::Format::D32_SFLOAT;
}

pub mod shadowmap_renderpass_consts {
    use ash::vk;

    pub const SHADOW_MAP_RESOLUTION: u32 = 2048;
    pub const DEPTH_CLEAR_VALUE: vk::ClearValue = vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue {
            depth: 1.0,
            stencil: 0,
        },
    };

    pub const SHADOW_MAP_DIMENSIONS: vk::Rect2D = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D {
            width: SHADOW_MAP_RESOLUTION,
            height: SHADOW_MAP_RESOLUTION,
        },
    };
}

#[derive(Clone)]
pub struct ShadowMapRenderPassOutput {
    pub depth: RenderPassAttachmentOutput,
}

pub struct ShadowMapRenderPass {
    // TODO: ref to device?
    device: Device,
    depth_image: Image,
    shadow_map_material: ShadowMapMaterial,
}

impl ShadowMapRenderPass {
    pub fn new(
        device: Device,
        allocator: &mut Allocator,
        pipeline_manager: &PipelineManager,
    ) -> Self {
        let depth_image = Self::create_depth_image(&device, allocator);

        Self {
            device,
            depth_image,
            shadow_map_material: pipeline_manager.shadow_map_material.clone(),
        }
    }

    pub fn destroy(&mut self, allocator: &mut Allocator) {
        self.depth_image.destroy(&self.device, allocator);
    }

    pub fn render(&self, command_buffer: vk::CommandBuffer, draw_data: &DrawData) {
        self.begin_render_pass(command_buffer);

        let view = draw_data.directional_light.get_view();
        let projection = draw_data.directional_light.get_projection();

        for draw_call in &draw_data.draw_calls {
            let push_data = PushConstantsData::new(&draw_call.model, &view, &projection);

            let buffers = [draw_call.mesh.positions_buffer];
            let offsets = [0, 0]; // TODO: Why tf 2?

            unsafe {
                self.device.cmd_push_constants(
                    command_buffer,
                    self.shadow_map_material.layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    push_data.get(),
                );

                self.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.shadow_map_material.pipeline,
                );

                self.device
                    .cmd_bind_vertex_buffers(command_buffer, 0, &buffers, &offsets);

                self.device.cmd_bind_index_buffer(
                    command_buffer,
                    draw_call.mesh.index_buffer,
                    0,
                    vk::IndexType::UINT32,
                );

                self.device.cmd_draw_indexed(
                    command_buffer,
                    draw_call.mesh.index_count,
                    1,
                    0,
                    0,
                    0,
                );
            };
        }

        self.end_render_pass(command_buffer);
    }

    fn begin_render_pass(&self, command_buffer: vk::CommandBuffer) {
        let depth_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(self.depth_image.image_view)
            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .resolve_mode(vk::ResolveModeFlags::NONE)
            .resolve_image_view(vk::ImageView::null())
            .resolve_image_layout(vk::ImageLayout::UNDEFINED)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(DEPTH_CLEAR_VALUE);

        command_buffer_helpers::single_image_pipeline_barrier(
            &self.device,
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            vk::AccessFlags::NONE,
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            self.depth_image.image,
            vk::ImageAspectFlags::DEPTH,
        );

        let rendering_info = vk::RenderingInfo::default()
            .render_area(SHADOW_MAP_DIMENSIONS)
            .layer_count(1)
            .view_mask(0)
            .color_attachments(&[])
            .depth_attachment(&depth_attachment);
        //.stencil_attachment();

        unsafe {
            self.device
                .cmd_begin_rendering(command_buffer, &rendering_info);
        }
    }

    fn end_render_pass(&self, command_buffer: vk::CommandBuffer) {
        unsafe { self.device.cmd_end_rendering(command_buffer) };
    }

    fn create_depth_image(device: &Device, allocator: &mut Allocator) -> Image {
        let create_info = &ImageCreateInfo {
            extent: vk::Extent3D::default()
                .width(
                    shadowmap_renderpass_consts::SHADOW_MAP_DIMENSIONS
                        .extent
                        .width,
                )
                .height(
                    shadowmap_renderpass_consts::SHADOW_MAP_DIMENSIONS
                        .extent
                        .height,
                )
                .depth(1),
            image_type: vk::ImageType::TYPE_2D,
            format: deferred_renderpass_consts::DEPTH,
            usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            view_type: vk::ImageViewType::TYPE_2D,
            aspect_mask: vk::ImageAspectFlags::DEPTH,
        };

        Image::new(device, allocator, create_info)
    }

    pub fn get_output(&self) -> ShadowMapRenderPassOutput {
        ShadowMapRenderPassOutput {
            depth: RenderPassAttachmentOutput {
                image: self.depth_image.image,
                image_view: self.depth_image.image_view,
                image_layout: vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
            },
        }
    }
}
