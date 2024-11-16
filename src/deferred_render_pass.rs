use std::clone;

use ash::{vk, Device};
use gpu_allocator::vulkan::Allocator;

use crate::{
    command_buffer_helpers,
    draw_data::DrawData,
    image::{Image, ImageCreateInfo},
    push_constants_data::PushConstantsData,
    shadow_map_render_pass::deferred_renderpass_consts::{self},
};

#[derive(Clone)]
pub struct RenderPassAttachmentOutput {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub image_layout: vk::ImageLayout, // = vk::ImageLayout::eUndefined;
}

#[derive(Clone)]
pub struct DeferredRenderPassOutput {
    pub color: RenderPassAttachmentOutput,
    pub normal: RenderPassAttachmentOutput,
    pub position: RenderPassAttachmentOutput,
    pub depth: RenderPassAttachmentOutput,
}

pub struct DeferredRenderPass {
    device: Device,
    render_area: vk::Rect2D,
    color_image: Image,
    normal_image: Image,
    position_image: Image,
    depth_image: Image,
}
const CLEAR_VALUE: vk::ClearValue = vk::ClearValue {
    color: vk::ClearColorValue {
        float32: [0.0, 0.0, 0.0, 0.0],
    },
};
const DEPTH_CLEAR_VALUE: vk::ClearValue = vk::ClearValue {
    depth_stencil: vk::ClearDepthStencilValue {
        depth: 1.0,
        stencil: 0,
    },
};

impl DeferredRenderPass {
    pub fn new(device: &Device, allocator: &mut Allocator, render_area: &vk::Rect2D) -> Self {
        Self {
            device: device.clone(),
            render_area: render_area.clone(),
            color_image: Self::create_color_image(
                device,
                allocator,
                render_area,
                deferred_renderpass_consts::COLOR,
            ),
            normal_image: Self::create_color_image(
                device,
                allocator,
                render_area,
                deferred_renderpass_consts::NORMAL,
            ),
            position_image: Self::create_color_image(
                device,
                allocator,
                render_area,
                deferred_renderpass_consts::POSITION,
            ),
            depth_image: Self::create_depth_image(device, allocator, render_area),
        }
    }

    pub fn destroy(&mut self, allocator: &mut Allocator) {
        self.color_image.destroy(&self.device, allocator);
        self.normal_image.destroy(&self.device, allocator);
        self.position_image.destroy(&self.device, allocator);
        self.depth_image.destroy(&self.device, allocator);
    }

    pub fn render(&self, command_buffer: vk::CommandBuffer, draw_data: &DrawData) {
        self.begin_render_pass(command_buffer);

        // TODO: Why tf is deferredPipelineLayout a part of DrawData

        for draw_call in &draw_data.draw_calls {
            let push_data =
                PushConstantsData::new(&draw_call.model, &draw_data.view, &draw_data.projection);
            let buffers = [
                draw_call.mesh.positions_buffer,
                draw_call.mesh.attributes_buffer,
            ];
            let offsets = [0, 0];

            unsafe {
                self.device.cmd_push_constants(
                    command_buffer,
                    draw_data.deferred_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    push_data.get(),
                );

                self.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    draw_call.pipeline,
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
        let color_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(self.color_image.image_view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .resolve_mode(vk::ResolveModeFlags::NONE)
            .resolve_image_view(vk::ImageView::null())
            .resolve_image_layout(vk::ImageLayout::UNDEFINED)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(CLEAR_VALUE);

        let normal_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(self.normal_image.image_view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .resolve_mode(vk::ResolveModeFlags::NONE)
            .resolve_image_view(vk::ImageView::null())
            .resolve_image_layout(vk::ImageLayout::UNDEFINED)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(CLEAR_VALUE);

        let position_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(self.position_image.image_view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .resolve_mode(vk::ResolveModeFlags::NONE)
            .resolve_image_view(vk::ImageView::null())
            .resolve_image_layout(vk::ImageLayout::UNDEFINED)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(CLEAR_VALUE);

        let depth_attachment = vk::RenderingAttachmentInfo::default()
            .image_view(self.depth_image.image_view)
            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .resolve_mode(vk::ResolveModeFlags::NONE)
            .resolve_image_view(vk::ImageView::null())
            .resolve_image_layout(vk::ImageLayout::UNDEFINED)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(DEPTH_CLEAR_VALUE);

        let attachments = [color_attachment, normal_attachment, position_attachment];

        // // TODO: Single vk::CmdPipelineBarrier

        command_buffer_helpers::single_image_pipeline_barrier(
            &self.device,
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags::NONE,
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            self.color_image.image,
            vk::ImageAspectFlags::COLOR,
        );

        command_buffer_helpers::single_image_pipeline_barrier(
            &self.device,
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags::NONE,
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            self.normal_image.image,
            vk::ImageAspectFlags::COLOR,
        );

        command_buffer_helpers::single_image_pipeline_barrier(
            &self.device,
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags::NONE,
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            self.position_image.image,
            vk::ImageAspectFlags::COLOR,
        );

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
            .render_area(self.render_area)
            .layer_count(1)
            .view_mask(0)
            .color_attachments(&attachments)
            .depth_attachment(&depth_attachment);

        unsafe {
            self.device
                .cmd_begin_rendering(command_buffer, &rendering_info);
        }
    }

    fn end_render_pass(&self, command_buffer: vk::CommandBuffer) {
        unsafe { self.device.cmd_end_rendering(command_buffer) };
    }

    fn create_color_image(
        device: &Device,
        allocator: &mut Allocator,
        render_area: &vk::Rect2D,
        format: vk::Format,
    ) -> Image {
        let create_info = ImageCreateInfo {
            extent: vk::Extent3D::default()
                .width(render_area.extent.width)
                .height(render_area.extent.height)
                .depth(1),
            image_type: vk::ImageType::TYPE_2D,
            format,
            usage: vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED,
            view_type: vk::ImageViewType::TYPE_2D,
            aspect_mask: vk::ImageAspectFlags::COLOR,
        };

        Image::new(device, allocator, &create_info)
    }

    fn create_depth_image(
        device: &Device,
        allocator: &mut Allocator,
        render_area: &vk::Rect2D,
    ) -> Image {
        let create_info = ImageCreateInfo {
            extent: vk::Extent3D::default()
                .width(render_area.extent.width)
                .height(render_area.extent.height)
                .depth(1),
            image_type: vk::ImageType::TYPE_2D,
            format: deferred_renderpass_consts::DEPTH,
            usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            view_type: vk::ImageViewType::TYPE_2D,
            aspect_mask: vk::ImageAspectFlags::DEPTH,
        };

        Image::new(device, allocator, &create_info)
    }

    pub fn get_output(&self) -> DeferredRenderPassOutput {
        let color = RenderPassAttachmentOutput {
            image: self.color_image.image,
            image_view: self.color_image.image_view,
            image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let normal = RenderPassAttachmentOutput {
            image: self.normal_image.image,
            image_view: self.normal_image.image_view,
            image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let position = RenderPassAttachmentOutput {
            image: self.position_image.image,
            image_view: self.position_image.image_view,
            image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let depth = RenderPassAttachmentOutput {
            image: self.depth_image.image,
            image_view: self.depth_image.image_view,
            image_layout: vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
        };

        DeferredRenderPassOutput {
            color,
            normal,
            position,
            depth,
        }
    }
}
