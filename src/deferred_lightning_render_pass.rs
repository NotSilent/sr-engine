use ash::{vk, Device};

use crate::{
    command_buffer_helpers,
    deferred_render_pass::DeferredRenderPassOutput,
    image::Image,
    pipeline_manager::{DeferredLightningMaterial, PipelineManager},
    push_constants_data::LightningPushConstantsData,
    shadow_map_render_pass::ShadowMapRenderPassOutput,
};

const SWAPCHAIN_CLEAR_VALUE: vk::ClearValue = vk::ClearValue {
    color: vk::ClearColorValue {
        float32: [0.0, 0.0, 0.0, 0.0],
    },
};

pub struct DeferredLightningRenderPass {
    device: Device,
    // TODO: As parameter to render?
    render_area: vk::Rect2D,
    deferred_render_pass_output: DeferredRenderPassOutput,
    shadow_map_render_pass_output: ShadowMapRenderPassOutput,
    material: DeferredLightningMaterial,
}

impl DeferredLightningRenderPass {
    pub fn new(
        device: &Device,
        pipeline_manager: &mut PipelineManager,
        render_area: &vk::Rect2D,
        deferred_render_pass_output: &DeferredRenderPassOutput,
        shadow_map_render_pass_output: &ShadowMapRenderPassOutput,
    ) -> Self {
        Self {
            device: device.clone(),
            render_area: *render_area,
            deferred_render_pass_output: deferred_render_pass_output.clone(),
            shadow_map_render_pass_output: shadow_map_render_pass_output.clone(),
            material: pipeline_manager.create_deferred_lightning_material(
                deferred_render_pass_output.color.image_view,
                deferred_render_pass_output.normal.image_view,
                deferred_render_pass_output.position.image_view,
                shadow_map_render_pass_output.depth.image_view,
            ),
        }
    }

    pub fn render(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_image: vk::Image,
        swapchain_image_view: vk::ImageView,
        light_space: &nalgebra::Matrix4<f32>,
        view_direction: &nalgebra::Vector3<f32>,
    ) {
        self.begin_render_pass(command_buffer, swapchain_image, swapchain_image_view);

        let push_data = LightningPushConstantsData::new(light_space, view_direction);

        unsafe {
            self.device.cmd_push_constants(
                command_buffer,
                self.material.layout,
                vk::ShaderStageFlags::FRAGMENT,
                0,
                push_data.get(),
            );

            self.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.material.pipeline,
            );

            self.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.material.layout,
                0,
                &[self.material.set],
                &[],
            );

            self.device.cmd_draw(command_buffer, 6, 1, 0, 0);
        };

        self.end_render_pass(command_buffer, swapchain_image);
    }

    fn begin_render_pass(
        &self,
        command_buffer: vk::CommandBuffer,
        swapchain_image: vk::Image,
        swapchain_image_view: vk::ImageView,
    ) {
        let swapchain_attachments = [vk::RenderingAttachmentInfo::default()
            .image_view(swapchain_image_view)
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .resolve_mode(vk::ResolveModeFlags::NONE)
            .resolve_image_view(vk::ImageView::null())
            .resolve_image_layout(vk::ImageLayout::UNDEFINED)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(SWAPCHAIN_CLEAR_VALUE)];

        command_buffer_helpers::single_image_pipeline_barrier(
            &self.device,
            command_buffer,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags::NONE,
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            swapchain_image,
            vk::ImageAspectFlags::COLOR,
        );

        command_buffer_helpers::single_image_pipeline_barrier(
            &self.device,
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::AccessFlags::NONE,
            vk::AccessFlags::SHADER_READ,
            self.deferred_render_pass_output.color.image_layout,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            self.deferred_render_pass_output.color.image,
            vk::ImageAspectFlags::COLOR,
        );

        command_buffer_helpers::single_image_pipeline_barrier(
            &self.device,
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::AccessFlags::NONE,
            vk::AccessFlags::SHADER_READ,
            self.deferred_render_pass_output.normal.image_layout,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            self.deferred_render_pass_output.normal.image,
            vk::ImageAspectFlags::COLOR,
        );

        command_buffer_helpers::single_image_pipeline_barrier(
            &self.device,
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::AccessFlags::NONE,
            vk::AccessFlags::SHADER_READ,
            self.deferred_render_pass_output.position.image_layout,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            self.deferred_render_pass_output.position.image,
            vk::ImageAspectFlags::COLOR,
        );

        command_buffer_helpers::single_image_pipeline_barrier(
            &self.device,
            command_buffer,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::AccessFlags::NONE,
            vk::AccessFlags::SHADER_READ,
            self.shadow_map_render_pass_output.depth.image_layout,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            self.shadow_map_render_pass_output.depth.image,
            vk::ImageAspectFlags::DEPTH,
        );

        let rendering_info = vk::RenderingInfo::default()
            .render_area(self.render_area)
            .layer_count(1)
            .view_mask(0)
            .color_attachments(&swapchain_attachments);
        //.depth_attachment(&depth_attachment);

        unsafe {
            self.device
                .cmd_begin_rendering(command_buffer, &rendering_info);
        }
    }

    fn end_render_pass(&self, command_buffer: vk::CommandBuffer, swapchain_image: vk::Image) {
        unsafe { self.device.cmd_end_rendering(command_buffer) };

        command_buffer_helpers::single_image_pipeline_barrier(
            &self.device,
            command_buffer,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            vk::AccessFlags::NONE,
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
            swapchain_image,
            vk::ImageAspectFlags::COLOR,
        );
    }
}
