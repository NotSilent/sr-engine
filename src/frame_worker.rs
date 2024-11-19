use ash::{khr::swapchain, vk, Device};
use gpu_allocator::vulkan::Allocator;

use crate::{
    deferred_lightning_render_pass::DeferredLightningRenderPass,
    deferred_render_pass::DeferredRenderPass, draw_data::DrawData, image::Image,
    pipeline_manager::PipelineManager, shadow_map_render_pass::ShadowMapRenderPass,
};

// TODO: Some helper library
fn create_fence(device: &Device, flags: vk::FenceCreateFlags) -> vk::Fence {
    let create_info = vk::FenceCreateInfo::default().flags(flags);

    unsafe { device.create_fence(&create_info, None).unwrap() }
}

fn create_semaphore(device: &Device) -> vk::Semaphore {
    let create_info = vk::SemaphoreCreateInfo::default();

    unsafe { device.create_semaphore(&create_info, None).unwrap() }
}

fn create_command_pool(device: &Device, queue_family_index: u32) -> vk::CommandPool {
    let create_info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::TRANSIENT)
        .queue_family_index(queue_family_index);

    unsafe { device.create_command_pool(&create_info, None).unwrap() }
}

struct Synchronization {
    queue_fence: vk::Fence,
    image_acquire_semaphore: vk::Semaphore,
    present_semaphore: vk::Semaphore,
}

impl Synchronization {
    fn new(device: &Device) -> Self {
        let queue_fence = create_fence(device, vk::FenceCreateFlags::SIGNALED);
        let image_acquire_semaphore = create_semaphore(device);
        let present_semaphore = create_semaphore(device);

        Self {
            queue_fence,
            image_acquire_semaphore,
            present_semaphore,
        }
    }

    fn destroy(&self, device: &Device) {
        unsafe {
            device
                .wait_for_fences(&[self.queue_fence], true, u64::MAX)
                .unwrap();
            device.destroy_fence(self.queue_fence, None);
            device.destroy_semaphore(self.image_acquire_semaphore, None);
            device.destroy_semaphore(self.present_semaphore, None);
        }
    }

    fn wait_queue(&self, device: &Device) {
        let fences = [self.queue_fence];

        unsafe {
            device.wait_for_fences(&fences, true, u64::MAX).unwrap();
            device.reset_fences(&fences).unwrap();
        };
    }

    fn replace_semaphores(
        &mut self,
        device: &Device,
        image_acquire: vk::Semaphore,
        present: vk::Semaphore,
    ) {
        unsafe {
            device.destroy_semaphore(self.image_acquire_semaphore, None);
            device.destroy_semaphore(self.present_semaphore, None);
        }

        self.image_acquire_semaphore = image_acquire;
        self.present_semaphore = present;
    }
}

pub struct FrameWorker {
    device: Device,
    // TODO: replace with Image
    swapchain_image: vk::Image,
    swapchain_image_view: vk::ImageView,
    synchronization: Synchronization,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    shadow_map_render_pass: ShadowMapRenderPass,
    deferred_render_pass: DeferredRenderPass,
    deferred_lightning_render_pass: DeferredLightningRenderPass,
}

impl FrameWorker {
    pub fn new(
        device: Device,
        allocator: &mut Allocator,
        pipeline_manager: &mut PipelineManager,
        swapchain_image: vk::Image,
        swapchain_image_view: vk::ImageView,
        queue_family_index: u32,
        render_area: &vk::Rect2D,
    ) -> Self {
        let command_pool = create_command_pool(&device, queue_family_index);

        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);

        let command_buffer = unsafe { device.allocate_command_buffers(&allocate_info).unwrap()[0] };

        let synchronization = Synchronization::new(&device);

        let shadow_map_render_pass =
            ShadowMapRenderPass::new(device.clone(), allocator, pipeline_manager);

        let deferred_render_pass = DeferredRenderPass::new(&device, allocator, render_area);

        let deferred_lightning_render_pass = DeferredLightningRenderPass::new(
            &device,
            pipeline_manager,
            render_area,
            &deferred_render_pass.get_output(),
            &shadow_map_render_pass.get_output(),
        );

        Self {
            device,
            swapchain_image,
            swapchain_image_view,
            command_pool,
            command_buffer,
            synchronization,
            shadow_map_render_pass,
            deferred_render_pass,
            deferred_lightning_render_pass,
        }
    }

    pub fn destroy(&mut self, allocator: &mut Allocator) {
        unsafe {
            self.synchronization.destroy(&self.device);

            self.device
                .reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty())
                .unwrap();
            self.device.destroy_command_pool(self.command_pool, None);
        }

        self.shadow_map_render_pass.destroy(allocator);
        self.deferred_render_pass.destroy(allocator);
    }

    pub fn draw(
        &mut self,
        swapchain_loader: &swapchain::Device,
        swapchain: vk::SwapchainKHR,
        present_fence: vk::Fence,
        image_index: u32,
        image_acquire_semaphore: vk::Semaphore,
        graphics_queue: vk::Queue,
        draw_data: &DrawData,
    ) {
        let present_semaphore = create_semaphore(&self.device);

        self.synchronization.wait_queue(&self.device);

        unsafe {
            self.device
                .reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty())
                .unwrap();
        }

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe {
            self.device
                .begin_command_buffer(self.command_buffer, &begin_info)
                .unwrap()
        };

        self.deferred_render_pass
            .render(self.command_buffer, draw_data);

        self.shadow_map_render_pass
            .render(self.command_buffer, draw_data);

        // const glm::vec3 viewDirection = glm::inverse(-drawData.view)[2];

        //let view = -draw_data.view;
        let view = draw_data.view.try_inverse().unwrap();
        let view_direction = nalgebra::Vector3::new(view[(2, 0)], view[(2, 1)], view[(2, 2)]);

        self.deferred_lightning_render_pass.render(
            self.command_buffer,
            self.swapchain_image,
            self.swapchain_image_view,
            &(draw_data.directional_light.get_projection()
                * draw_data.directional_light.get_view()),
            &view_direction,
        );

        unsafe { self.device.end_command_buffer(self.command_buffer).unwrap() };

        let image_acquire_semaphore_submit_infos = [vk::SemaphoreSubmitInfo::default()
            .semaphore(image_acquire_semaphore)
            .stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)];

        let command_buffer_submit_infos =
            [vk::CommandBufferSubmitInfo::default().command_buffer(self.command_buffer)];

        let present_semaphore_submit_infos = [vk::SemaphoreSubmitInfo::default()
            .semaphore(present_semaphore)
            .stage_mask(vk::PipelineStageFlags2::BOTTOM_OF_PIPE)];

        let submit_info = vk::SubmitInfo2::default()
            .wait_semaphore_infos(&image_acquire_semaphore_submit_infos)
            .command_buffer_infos(&command_buffer_submit_infos)
            .signal_semaphore_infos(&present_semaphore_submit_infos);

        unsafe {
            self.device
                .queue_submit2(
                    graphics_queue,
                    &[submit_info],
                    self.synchronization.queue_fence,
                )
                .unwrap()
        };

        let present_semaphores = [present_semaphore];
        let swapchains = [swapchain];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::default()
            .wait_semaphores(&present_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices);

        unsafe {
            swapchain_loader
                .queue_present(graphics_queue, &present_info)
                .unwrap()
        };

        let present_fences = [present_fence];
        unsafe {
            self.device
                .wait_for_fences(&present_fences, true, u64::MAX)
                .unwrap();
            self.device.reset_fences(&present_fences).unwrap();
        };

        self.synchronization.replace_semaphores(
            &self.device,
            image_acquire_semaphore,
            present_semaphore,
        );
    }
}
