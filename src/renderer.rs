use crate::buffer::VulkanResource;
use crate::camera::Camera;
use crate::command_buffer::CommandBuffer;
use crate::mesh::Mesh;
use crate::pipeline::Pipeline;
use crate::push_constants_data::PushConstantsData;
use crate::vertex::Vertex;
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::{vk, Device, Entry, Instance};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use gpu_allocator::AllocatorDebugSettings;
use nalgebra::{Matrix4, RealField};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::borrow::Cow;
use std::ffi::CStr;
use winit::window::Window;

pub struct Renderer {
    _entry: Entry,
    _instance: Instance,
    _debug_utils: DebugUtils,
    device: Device,
    surface_loader: Surface,
    swapchain_loader: Swapchain,

    surface: vk::SurfaceKHR,
    _surface_format: vk::SurfaceFormatKHR,
    swapchain: vk::SwapchainKHR,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,

    _physical_device: vk::PhysicalDevice,

    _debug_utils_messenger: vk::DebugUtilsMessengerEXT,
    allocator: std::mem::ManuallyDrop<Allocator>, // Could be Option?

    graphics_queue_family_index: u32,
    graphics_queue: vk::Queue,

    command_pool: vk::CommandPool,

    render_area: vk::Rect2D,

    descriptor_set_layout: vk::DescriptorSetLayout,
    //descriptor_set: vk::DescriptorSet,
    pipeline_layout: vk::PipelineLayout,
    pipeline: Pipeline,

    camera: Camera,
    mesh: Mesh,
}

impl Renderer {
    unsafe extern "system" fn vulkan_debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _user_data: *mut std::os::raw::c_void,
    ) -> vk::Bool32 {
        let callback_data = *p_callback_data;
        let message_id_number = callback_data.message_id_number;

        let message_id_name = if callback_data.p_message_id_name.is_null() {
            Cow::from("")
        } else {
            CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
        };

        let message = if callback_data.p_message.is_null() {
            Cow::from("")
        } else {
            CStr::from_ptr(callback_data.p_message).to_string_lossy()
        };

        println!(
            "{:?}:\n{:?} [{} ({})] : {}\n",
            message_severity, message_type, message_id_name, message_id_number, message,
        );

        vk::FALSE
    }

    fn create_debug_utils(entry: &Entry, instance: &Instance) -> DebugUtils {
        DebugUtils::new(entry, instance)
    }

    fn create_debug_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING, //| vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION, //| vk::DebugUtilsMessageTypeFlagsEXT::GENERAL,
            )
            .pfn_user_callback(Some(Self::vulkan_debug_callback))
            .build()
    }

    fn create_debug_utils_messenger(debug_utils: &DebugUtils) -> vk::DebugUtilsMessengerEXT {
        let debug_info = Self::create_debug_info();

        unsafe {
            debug_utils
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        }
    }

    fn create_instance(entry: &Entry, window: &Window) -> Instance {
        let app_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"VulkanTriangle\0") };
        let application_info = vk::ApplicationInfo::builder()
            .application_version(0)
            .application_name(app_name)
            .api_version(vk::make_api_version(0, 1, 3, 0))
            .engine_version(0)
            .engine_name(app_name)
            .build();

        let layer_names = unsafe {
            [CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0").as_ptr()]
        };
        let mut extension_names =
            ash_window::enumerate_required_extensions(window.raw_display_handle())
                .unwrap()
                .to_vec();
        extension_names.push(DebugUtils::name().as_ptr());

        let mut debug_info = Self::create_debug_info();

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&extension_names)
            .push_next(&mut debug_info)
            .build();

        unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
    }

    fn select_physical_device(instance: &Instance) -> vk::PhysicalDevice {
        let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };
        *physical_devices.get(0).unwrap()
    }

    fn create_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        graphics_queue_family_index: u32,
    ) -> Device {
        let device_queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue_family_index)
            .queue_priorities(&[1.0_f32])
            .build();

        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::builder()
            .dynamic_rendering(true)
            .build();

        let device_create_info = vk::DeviceCreateInfo::builder()
            .push_next(&mut vulkan_13_features)
            .queue_create_infos(&[device_queue_create_info])
            .enabled_extension_names(&[Swapchain::name().as_ptr()])
            .build();

        unsafe {
            instance
                .create_device(physical_device, &device_create_info, None)
                .unwrap()
        }
    }

    fn create_surface(entry: &Entry, instance: &Instance, window: &Window) -> vk::SurfaceKHR {
        unsafe {
            ash_window::create_surface(
                entry,
                instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
            .unwrap()
        }
    }

    fn create_swapchain(
        swapchain: &Swapchain,
        surface: vk::SurfaceKHR,
        surface_format: vk::SurfaceFormatKHR,
        render_area: vk::Rect2D,
    ) -> vk::SwapchainKHR {
        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(2)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(render_area.extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            //.queue_family_indices(&[])
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY) // TODO: from surface
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE) // TODO: from surface?
            .present_mode(vk::PresentModeKHR::IMMEDIATE)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null())
            .build();

        unsafe { swapchain.create_swapchain(&create_info, None).unwrap() }
    }

    fn create_allocator(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
    ) -> Allocator {
        let debug_settings = AllocatorDebugSettings {
            log_allocations: true,
            log_frees: true,
            log_leaks_on_shutdown: true,
            log_stack_traces: true,
            log_memory_information: true,
            store_stack_traces: true,
        };

        Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: device.clone(),
            physical_device,
            debug_settings,
            buffer_device_address: false,
        })
        .unwrap()
    }

    fn get_graphics_queue_family_index(
        queue_families: &[vk::QueueFamilyProperties],
    ) -> Option<u32> {
        // TODO: rewrite without for (find_map? need index)?
        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                return Some(i as u32);
            }
        }

        None
    }

    fn create_command_pool(device: &Device, graphics_queue_family_index: u32) -> vk::CommandPool {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(graphics_queue_family_index)
            .build();
        unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .unwrap()
        }
    }

    pub fn new(window: &Window) -> Self {
        let width = 1920;
        let height = 1080;
        let render_area = vk::Rect2D::builder()
            .extent(vk::Extent2D::builder().width(width).height(height).build())
            .build();

        let entry = Entry::linked();
        let instance = Self::create_instance(&entry, window);
        let debug_utils = Self::create_debug_utils(&entry, &instance);
        let debug_utils_messenger = Self::create_debug_utils_messenger(&debug_utils);
        let physical_device = Self::select_physical_device(&instance);

        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let graphics_queue_family_index =
            Self::get_graphics_queue_family_index(&queue_families).unwrap();

        let device = Self::create_device(&instance, physical_device, graphics_queue_family_index);

        let surface_loader = Surface::new(&entry, &instance);
        let swapchain_loader = Swapchain::new(&instance, &device);

        let surface = Self::create_surface(&entry, &instance, window);
        let surface_format = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .unwrap()[0]
        };

        let swapchain =
            Self::create_swapchain(&swapchain_loader, surface, surface_format, render_area);

        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        let swapchain_image_views = swapchain_images
            .iter()
            .map(|&image| {
                let image_view_create_info = vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(
                        vk::ComponentMapping::builder()
                            .r(vk::ComponentSwizzle::R)
                            .g(vk::ComponentSwizzle::G)
                            .b(vk::ComponentSwizzle::B)
                            .a(vk::ComponentSwizzle::A)
                            .build(),
                    )
                    .subresource_range(
                        vk::ImageSubresourceRange::builder()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1)
                            .build(),
                    )
                    .build();

                unsafe {
                    device
                        .create_image_view(&image_view_create_info, None)
                        .unwrap()
                }
            })
            .collect();

        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family_index, 0) };

        let mut allocator = Self::create_allocator(&instance, &device, physical_device);

        let command_pool = Self::create_command_pool(&device, graphics_queue_family_index);

        let descriptor_set_layout = Self::create_descriptor_set_layout(&device);
        //let descriptor_set = vk::DescriptorSet::default(); //Self::create_descriptor_set();

        let pipeline_layout = Self::create_pipeline_layout(&device, &[descriptor_set_layout]);
        let pipeline = Pipeline::new(
            &device,
            pipeline_layout,
            render_area,
            &[surface_format.format],
        );

        let mesh = Mesh::new(
            &device,
            &mut allocator,
            graphics_queue,
            command_pool,
            "sphere",
            vec![
                Vertex::new(-0.5, 0.0, 0.5),
                Vertex::new(-0.5, 0.0, -0.5),
                Vertex::new(0.5, 0.0, 0.5),
                Vertex::new(0.5, 0.0, -0.5),
            ],
            vec![0, 1, 2, 2, 1, 3],
        );

        Self {
            _entry: entry,
            _instance: instance,
            _debug_utils: debug_utils,
            device,
            surface_loader,
            swapchain_loader,
            surface,
            _surface_format: surface_format,
            swapchain,
            swapchain_images,
            swapchain_image_views,
            _physical_device: physical_device,
            _debug_utils_messenger: debug_utils_messenger,
            allocator: std::mem::ManuallyDrop::new(allocator),
            graphics_queue_family_index,
            graphics_queue,
            command_pool,
            render_area,
            descriptor_set_layout,
            //descriptor_set,
            pipeline_layout,
            pipeline,
            camera: Camera::new(
                5.0,
                -10.0,
                5.0,
                width as f32,
                height as f32,
                f32::pi() / 2.0,
                0.1,
                100.0,
            ),
            mesh,
        }
    }

    fn create_semaphore(&self) -> vk::Semaphore {
        let create_info = vk::SemaphoreCreateInfo::builder().build();

        unsafe { self.device.create_semaphore(&create_info, None).unwrap() }
    }

    fn create_fence(&self) -> vk::Fence {
        let create_info = vk::FenceCreateInfo::builder().build();

        unsafe { self.device.create_fence(&create_info, None).unwrap() }
    }

    fn create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
        let create_info = vk::DescriptorSetLayoutCreateInfo::default();

        unsafe {
            device
                .create_descriptor_set_layout(&create_info, None)
                .unwrap()
        }
    }

    fn create_pipeline_layout(
        device: &Device,
        descriptor_set_layouts: &[vk::DescriptorSetLayout],
    ) -> vk::PipelineLayout {
        let create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(descriptor_set_layouts)
            .push_constant_ranges(&[vk::PushConstantRange::builder()
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .offset(0)
                .size(std::mem::size_of::<PushConstantsData>() as u32)
                .build()])
            .build();

        unsafe { device.create_pipeline_layout(&create_info, None).unwrap() }
    }

    fn record_command_buffer(
        &self,
        image: vk::Image,
        image_view: vk::ImageView,
    ) -> vk::CommandBuffer {
        let create_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .command_buffer_count(1)
            .build();

        let command_buffers =
            unsafe { self.device.allocate_command_buffers(&create_info).unwrap() };
        let cmd = command_buffers[0];

        let color_attachments = [vk::RenderingAttachmentInfo::builder()
            .image_view(image_view)
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .resolve_mode(vk::ResolveModeFlags::NONE)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.5, 0.0, 0.5, 1.0],
                },
            })
            .build()];

        let rendering_info = vk::RenderingInfo::builder()
            .render_area(self.render_area)
            .layer_count(1)
            .color_attachments(&color_attachments)
            .build();

        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
            .build();

        unsafe {
            self.device.begin_command_buffer(cmd, &begin_info).unwrap();

            CommandBuffer::pipeline_barrier(
                &self.device,
                cmd,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::AccessFlags::NONE,
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                self.graphics_queue_family_index,
                self.graphics_queue_family_index,
                image,
                vk::ImageAspectFlags::COLOR,
            );

            self.device.cmd_begin_rendering(cmd, &rendering_info);

            let model = Matrix4::identity();
            let push_constants_data = PushConstantsData::new(
                &model,
                &self.camera.get_view(),
                self.camera.get_projection(),
            );

            self.device.cmd_push_constants(
                cmd,
                self.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                push_constants_data.get(),
            );

            //self.device.cmd_bind_vertex_buffers(self.triangle_buffer);

            self.device.cmd_bind_pipeline(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.get(),
            );

            self.device.cmd_bind_vertex_buffers(
                cmd,
                0,
                &[self.mesh.get_vertex_buffer().buffer],
                &[0],
            );
            self.device.cmd_bind_index_buffer(
                cmd,
                self.mesh.get_index_buffer().buffer,
                0,
                vk::IndexType::UINT16,
            );

            self.device.cmd_draw_indexed(cmd, 6, 1, 0, 0, 0);

            self.device.cmd_end_rendering(cmd);

            CommandBuffer::pipeline_barrier(
                &self.device,
                cmd,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                vk::AccessFlags::NONE,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk::ImageLayout::PRESENT_SRC_KHR,
                self.graphics_queue_family_index,
                self.graphics_queue_family_index,
                image,
                vk::ImageAspectFlags::COLOR,
            );

            self.device.end_command_buffer(cmd).unwrap();
        }

        cmd
    }

    pub fn render(&self) {
        unsafe {
            let acquire_image_semaphore = self.create_semaphore();
            let queue_submit_semaphore = self.create_semaphore();

            let fence = self.create_fence();

            let image_indices = self
                .swapchain_loader
                .acquire_next_image(
                    self.swapchain,
                    u64::MAX,
                    acquire_image_semaphore,
                    vk::Fence::null(),
                )
                .unwrap()
                .0;

            let cmd = self.record_command_buffer(
                self.swapchain_images[image_indices as usize],
                self.swapchain_image_views[image_indices as usize],
            );

            let submit_info = vk::SubmitInfo::builder()
                .command_buffers(&[cmd])
                .signal_semaphores(&[queue_submit_semaphore])
                .wait_semaphores(&[acquire_image_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .build();

            let present_info = vk::PresentInfoKHR::builder()
                .wait_semaphores(&[queue_submit_semaphore])
                .swapchains(&[self.swapchain])
                .image_indices(&[image_indices])
                .build();

            self.device
                .queue_submit(self.graphics_queue, &[submit_info], fence)
                .expect("Couldn't submit");

            self.swapchain_loader
                .queue_present(self.graphics_queue, &present_info)
                .expect("Couldn't present");

            self.device
                .wait_for_fences(&[fence], true, u64::MAX)
                .unwrap();

            self.device.destroy_semaphore(acquire_image_semaphore, None);
            self.device.destroy_semaphore(queue_submit_semaphore, None);
            self.device.destroy_fence(fence, None);

            self.device.free_command_buffers(self.command_pool, &[cmd]);
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        self.mesh.release(&self.device, &mut self.allocator);

        println!("{:?}", &self.allocator);

        unsafe {
            self.device.destroy_pipeline(self.pipeline.get(), None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);

            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);

            for &image_view in self.swapchain_image_views.iter() {
                self.device.destroy_image_view(image_view, None);
            }

            self.device.destroy_command_pool(self.command_pool, None);

            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None);
            self.surface_loader.destroy_surface(self.surface, None);

            std::mem::ManuallyDrop::drop(&mut self.allocator);

            self.device.destroy_device(None);
            self._debug_utils
                .destroy_debug_utils_messenger(self._debug_utils_messenger, None);
            self._instance.destroy_instance(None);
        }
    }
}
