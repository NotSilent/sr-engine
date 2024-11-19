use crate::buffer_manager::BufferManager;
use crate::camera::Camera;
use crate::draw_data::{DrawCall, DrawData, MeshData};
use crate::frame_worker::FrameWorker;
use crate::patched_sphere::PatchedSphere;
use crate::pipeline::Pipeline;
use crate::pipeline_manager::PipelineManager;
use crate::push_constants_data::PushConstantsData;
use ash::ext::debug_utils;
use ash::khr::{surface, swapchain};
use ash::{vk, Device, Entry, Instance};
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use nalgebra::{Matrix4, RealField};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::borrow::Cow;
use std::ffi::CStr;
use winit::window::Window;

pub struct Renderer {
    _entry: Entry,
    _instance: Instance,
    _debug_utils: debug_utils::Instance,
    device: Device,
    surface_loader: surface::Instance,
    swapchain_loader: swapchain::Device,

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
    sphere: PatchedSphere,

    frame_workers: Vec<FrameWorker>,
    pipeline_manager: PipelineManager,

    buffer_manager: BufferManager,
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
            "[{:?}] [{:?}] [{} ({})] : {}",
            message_severity, message_type, message_id_name, message_id_number, message,
        );

        vk::FALSE
    }

    fn create_debug_utils(entry: &Entry, instance: &Instance) -> debug_utils::Instance {
        debug_utils::Instance::new(entry, instance)
    }

    // TODO: static?
    fn create_debug_info() -> vk::DebugUtilsMessengerCreateInfoEXT<'static> {
        vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING, //| vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION, //| vk::DebugUtilsMessageTypeFlagsEXT::GENERAL,
            )
            .pfn_user_callback(Some(Self::vulkan_debug_callback))
    }

    fn create_debug_utils_messenger(
        debug_utils: &debug_utils::Instance,
    ) -> vk::DebugUtilsMessengerEXT {
        let debug_info = Self::create_debug_info();

        unsafe {
            debug_utils
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        }
    }

    fn create_instance(entry: &Entry, window: &Window) -> Instance {
        let app_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"VulkanTriangle\0") };
        let application_info = vk::ApplicationInfo::default()
            .application_version(0)
            .application_name(app_name)
            .api_version(vk::make_api_version(0, 1, 3, 0))
            .engine_version(0)
            .engine_name(app_name);

        let layer_names = unsafe {
            [CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0").as_ptr()]
        };
        let mut extension_names =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                .unwrap()
                .to_vec();
        extension_names.push(debug_utils::NAME.as_ptr());

        let mut debug_info = Self::create_debug_info();

        let instance_create_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&extension_names)
            .push_next(&mut debug_info);

        unsafe { entry.create_instance(&instance_create_info, None).unwrap() }
    }

    fn select_physical_device(instance: &Instance) -> vk::PhysicalDevice {
        let physical_devices = unsafe { instance.enumerate_physical_devices().unwrap() };
        *physical_devices.first().unwrap()
    }

    fn create_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        graphics_queue_family_index: u32,
    ) -> Device {
        let device_queue_create_info = vk::DeviceQueueCreateInfo::default()
            .queue_family_index(graphics_queue_family_index)
            .queue_priorities(&[1.0_f32]);

        // For gpu-allocator
        let mut vulkan_12_features =
            vk::PhysicalDeviceVulkan12Features::default().buffer_device_address(true);

        let mut vulkan_13_features = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);

        let device_queue_create_infos = [device_queue_create_info];

        let enabled_extension_names = [swapchain::NAME.as_ptr()];

        let device_create_info = vk::DeviceCreateInfo::default()
            .push_next(&mut vulkan_12_features)
            .push_next(&mut vulkan_13_features)
            .queue_create_infos(&device_queue_create_infos)
            .enabled_extension_names(&enabled_extension_names);

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
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .unwrap()
        }
    }

    fn create_swapchain(
        swapchain: &swapchain::Device,
        surface: vk::SurfaceKHR,
        surface_format: vk::SurfaceFormatKHR,
        surface_transform: vk::SurfaceTransformFlagsKHR,
        render_area: vk::Rect2D,
        queue_family_indices: &[u32],
    ) -> vk::SwapchainKHR {
        let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(3)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(render_area.extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .queue_family_indices(queue_family_indices)
            .pre_transform(surface_transform) // TODO: from surface
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(vk::PresentModeKHR::FIFO)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        unsafe { swapchain.create_swapchain(&create_info, None).unwrap() }
    }

    fn create_allocator(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
    ) -> Allocator {
        Allocator::new(&AllocatorCreateDesc {
            // TODO: clone?
            instance: instance.clone(),
            device: device.clone(),
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
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
        let command_pool_create_info =
            vk::CommandPoolCreateInfo::default().queue_family_index(graphics_queue_family_index);
        unsafe {
            device
                .create_command_pool(&command_pool_create_info, None)
                .unwrap()
        }
    }

    pub fn new(window: &Window) -> Self {
        let width = 1920;
        let height = 1080;
        let render_area =
            vk::Rect2D::default().extent(vk::Extent2D::default().width(width).height(height));

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

        let surface_loader = surface::Instance::new(&entry, &instance);
        let swapchain_loader = swapchain::Device::new(&instance, &device);

        let surface = Self::create_surface(&entry, &instance, window);
        let surface_format = unsafe {
            surface_loader
                .get_physical_device_surface_formats(physical_device, surface)
                .unwrap()[0]
        };

        let surface_capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)
        }
        .unwrap();

        let swapchain = Self::create_swapchain(
            &swapchain_loader,
            surface,
            surface_format,
            surface_capabilities.current_transform,
            render_area,
            &[graphics_queue_family_index],
        );

        let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(swapchain).unwrap() };
        let swapchain_image_views: Vec<vk::ImageView> = swapchain_images
            .iter()
            .map(|&image| {
                let image_view_create_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
                    .components(
                        vk::ComponentMapping::default()
                            .r(vk::ComponentSwizzle::R)
                            .g(vk::ComponentSwizzle::G)
                            .b(vk::ComponentSwizzle::B)
                            .a(vk::ComponentSwizzle::A),
                    )
                    .subresource_range(
                        vk::ImageSubresourceRange::default()
                            .aspect_mask(vk::ImageAspectFlags::COLOR)
                            .base_mip_level(0)
                            .level_count(1)
                            .base_array_layer(0)
                            .layer_count(1),
                    );

                unsafe {
                    device
                        .create_image_view(&image_view_create_info, None)
                        .unwrap()
                }
            })
            .collect();

        let mut allocator = Self::create_allocator(&instance, &device, physical_device);

        let mut pipeline_manager =
            PipelineManager::new(device.clone(), surface_format.format, render_area);

        // TODO: Remove device clone
        let frame_workers = swapchain_images
            .iter()
            .zip(swapchain_image_views.iter())
            .map(|(&image, &image_view)| {
                FrameWorker::new(
                    device.clone(),
                    &mut allocator,
                    &mut pipeline_manager,
                    image,
                    image_view,
                    graphics_queue_family_index,
                    &render_area,
                )
            })
            .collect();

        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family_index, 0) };
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

        let mut buffer_manager = BufferManager::new(&device, command_pool);

        let sphere = PatchedSphere::new(3);

        buffer_manager.add_buffer(
            "sphereIndices",
            &mut allocator,
            graphics_queue,
            &sphere.indices,
            vk::BufferUsageFlags::INDEX_BUFFER,
        );

        buffer_manager.add_buffer(
            "sphereVertices",
            &mut allocator,
            graphics_queue,
            &sphere.positions,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        );

        buffer_manager.add_buffer(
            "sphereNormals",
            &mut allocator,
            graphics_queue,
            &sphere.normals,
            vk::BufferUsageFlags::VERTEX_BUFFER,
        );

        // renderer->addBuffer("planeIndices", VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
        //         planeIndices.size() * sizeof(uint32_t), planeIndices.data());
        // renderer->addBuffer("planeVertices", VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
        //         planePositions.size() * sizeof(glm::vec3),
        //         planePositions.data());
        // renderer->addBuffer("planeNormals", VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
        //         planeNormals.size() * sizeof(glm::vec3),
        //         planeNormals.data());

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
                0.0,
                0.0,
                0.0,
                width as f32,
                height as f32,
                f32::pi() / 2.0,
                0.1,
                100.0,
            ),
            sphere,
            frame_workers,
            pipeline_manager,
            buffer_manager,
        }
    }

    // TODO: Some helper library
    fn create_semaphore(&self) -> vk::Semaphore {
        let create_info = vk::SemaphoreCreateInfo::default();

        unsafe { self.device.create_semaphore(&create_info, None).unwrap() }
    }

    fn create_fence(&self) -> vk::Fence {
        let create_info = vk::FenceCreateInfo::default();

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
        let push_constants = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<PushConstantsData>() as u32)];

        let create_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(descriptor_set_layouts)
            .push_constant_ranges(&push_constants);

        unsafe { device.create_pipeline_layout(&create_info, None).unwrap() }
    }

    pub fn render(&mut self) {
        let present_fence = unsafe {
            self.device
                .create_fence(&vk::FenceCreateInfo::default(), None)
                .unwrap()
        };

        let acquire_image_semaphore = self.create_semaphore();

        let acquire_image_result = unsafe {
            self.swapchain_loader.acquire_next_image(
                self.swapchain,
                u64::MAX,
                acquire_image_semaphore,
                present_fence,
            )
        };

        // match acquire_image_result {
        //     Ok((image_index, b)) => {
        //         println!("{}: {}", image_index, b);
        //     }
        //     Err(error) => {
        //         println!("{}", error);
        //     }
        // }

        let (next_image, _) = acquire_image_result.expect("Acquiring next image failed");

        let mut draw_data =
            DrawData::new(&self.camera, self.pipeline_manager.deferred_pipeline_layout);

        let pipeline = self.pipeline_manager.deferred_pipeline;

        let sphere_mesh = MeshData::new(
            self.sphere.indices.len() as u32,
            self.buffer_manager.get_buffer("sphereIndices").buffer,
            self.buffer_manager.get_buffer("sphereVertices").buffer,
            self.buffer_manager.get_buffer("sphereNormals").buffer,
        );

        let model = Matrix4::identity().append_translation(&nalgebra::Vector3::new(0.0, 0.0, 0.0));

        let sphere_draw_call = DrawCall::new(&sphere_mesh, model, pipeline);
        draw_data.add_draw_call(sphere_draw_call);

        if let Some(frame_worker) = self.frame_workers.get_mut(next_image as usize) {
            frame_worker.draw(
                &self.swapchain_loader,
                self.swapchain,
                present_fence,
                next_image,
                acquire_image_semaphore,
                self.graphics_queue,
                &draw_data,
            );
        }

        unsafe { self.device.destroy_fence(present_fence, None) };
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        println!("{:?}", &self.allocator);

        unsafe {
            //self.device.device_wait_idle().unwrap();

            // self.device
            //     .wait_for_fences(&[self.fence], true, u64::MAX)
            //     .unwrap();
            // self.device.destroy_fence(self.fence, None);

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
