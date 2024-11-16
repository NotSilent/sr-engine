// TODO: Separate pipelines per renderpass?
// Doesn't make much sense for shadowmaps and final composition pipeline to be here
// since they are unchanged and only used by those renderpasses

use std::ffi::CStr;

use ash::{vk, Device};

use crate::{
    push_constants_data::{LightningPushConstantsData, PushConstantsData},
    shader_manager::{Shader, ShaderManager},
    shadow_map_render_pass::{deferred_renderpass_consts, shadowmap_renderpass_consts},
};

pub struct DeferredLightningMaterial {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub set: vk::DescriptorSet,
}

#[derive(Clone)]
pub struct ShadowMapMaterial {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

pub struct PipelineManager {
    device: Device,
    render_area: vk::Rect2D,
    shader_manager: ShaderManager,
    default_sampler: vk::Sampler,
    descriptor_pool: vk::DescriptorPool,
    pub deferred_pipeline_layout: vk::PipelineLayout,
    deferred_pipeline: vk::Pipeline,
    deferred_lightning_descriptor_set_layout: vk::DescriptorSetLayout,
    deferred_lightning_pipeline_layout: vk::PipelineLayout,
    deferred_lightning_pipeline: vk::Pipeline,
    deferred_lightning_sets: Vec<vk::DescriptorSet>,
    pub shadow_map_material: ShadowMapMaterial,
}

impl PipelineManager {
    pub fn new(device: Device, swapchain_format: vk::Format, render_area: vk::Rect2D) -> Self {
        let mut shader_manager = ShaderManager::new(device.clone());

        let deferred_push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(std::mem::size_of::<PushConstantsData>() as u32);

        let deferred_lightning_push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(std::mem::size_of::<LightningPushConstantsData>() as u32);

        let default_sampler = Self::create_default_sampler(&device);

        let descriptor_pool = Self::create_descriptor_pool(&device);
        let deferred_lightning_descriptor_set_layout = Self::create_descriptor_set_layout(&device);
        let deferred_pipeline_layout =
            Self::create_pipeline_layout(&device, &[], &[deferred_push_constant_range]);
        let deferred_lightning_pipeline_layout = Self::create_pipeline_layout(
            &device,
            &[deferred_lightning_descriptor_set_layout],
            &[deferred_lightning_push_constant_range],
        );
        let deferred_pipeline = Self::create_deferred_pipeline(
            &device,
            &mut shader_manager,
            render_area,
            deferred_pipeline_layout,
        );
        let deferred_lightning_pipeline = Self::create_deferred_lightning_pipeline(
            &device,
            &mut shader_manager,
            render_area,
            swapchain_format,
            deferred_lightning_pipeline_layout,
        );

        // TODO: shadowmap pushrange/uniform
        let shadow_map_layout =
            Self::create_pipeline_layout(&device, &[], &[deferred_push_constant_range]);
        let shadow_map_material = ShadowMapMaterial {
            layout: shadow_map_layout,
            pipeline: Self::create_shadow_map_pipeline(
                &device,
                &mut shader_manager,
                shadowmap_renderpass_consts::SHADOW_MAP_DIMENSIONS,
                shadow_map_layout,
            ),
        };

        Self {
            device,
            render_area,
            shader_manager,
            default_sampler,
            descriptor_pool,
            deferred_pipeline_layout,
            deferred_pipeline,
            deferred_lightning_descriptor_set_layout,
            deferred_lightning_pipeline_layout,
            deferred_lightning_pipeline,
            deferred_lightning_sets: Vec::new(),
            shadow_map_material,
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.device
                .destroy_pipeline(self.shadow_map_material.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.shadow_map_material.layout, None);

            self.device.destroy_pipeline(self.deferred_pipeline, None);
            self.device
                .destroy_pipeline(self.deferred_lightning_pipeline, None);
            self.device
                .destroy_pipeline_layout(self.deferred_pipeline_layout, None);
            self.device
                .destroy_pipeline_layout(self.deferred_lightning_pipeline_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.deferred_lightning_descriptor_set_layout, None);
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device.destroy_sampler(self.default_sampler, None);
        }

        self.shader_manager.destroy();
    }

    fn create_default_sampler(device: &Device) -> vk::Sampler {
        let create_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
            .mip_lod_bias(0.0)
            .anisotropy_enable(false)
            .max_anisotropy(0.0)
            .compare_enable(false)
            .compare_op(vk::CompareOp::NEVER)
            .min_lod(0.0)
            .max_lod(0.0)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false);

        unsafe { device.create_sampler(&create_info, None).unwrap() }
    }

    pub fn create_deferred_lightning_material(
        &mut self,
        color: vk::ImageView,
        normal: vk::ImageView,
        position: vk::ImageView,
        shadow_map: vk::ImageView,
    ) -> DeferredLightningMaterial {
        let image_infos = [
            vk::DescriptorImageInfo {
                sampler: self.default_sampler,
                image_view: color,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
            vk::DescriptorImageInfo {
                sampler: self.default_sampler,
                image_view: normal,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
            vk::DescriptorImageInfo {
                sampler: self.default_sampler,
                image_view: position,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
            vk::DescriptorImageInfo {
                sampler: self.default_sampler,
                image_view: shadow_map,
                image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            },
        ];

        let set = self.create_deferred_lightning_set(
            self.descriptor_pool,
            self.deferred_lightning_descriptor_set_layout,
        );
        self.deferred_lightning_sets.push(set);

        let descriptor_write = vk::WriteDescriptorSet::default()
            .dst_set(set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_count(image_infos.len() as u32)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_infos);

        unsafe { self.device.update_descriptor_sets(&[descriptor_write], &[]) };

        DeferredLightningMaterial {
            layout: self.deferred_lightning_pipeline_layout,
            pipeline: self.deferred_lightning_pipeline,
            set,
        }
    }

    fn create_descriptor_pool(device: &Device) -> vk::DescriptorPool {
        // TODO: Configurable and per type
        // Currently should == swapchain images
        static DESCRIPTOR_SET_COUNT: u32 = 4;

        let descriptor_pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(DESCRIPTOR_SET_COUNT)];

        let create_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(DESCRIPTOR_SET_COUNT)
            .pool_sizes(&descriptor_pool_sizes);

        unsafe { device.create_descriptor_pool(&create_info, None).unwrap() }
    }

    fn create_descriptor_set_layout(device: &Device) -> vk::DescriptorSetLayout {
        // TODO: try immutable samplers for deferred lightning

        let bindings = [
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            //.immutable_samplers(&[]),
            vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            //.immutable_samplers(&[]),
            vk::DescriptorSetLayoutBinding::default()
                .binding(2)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            //.immutable_samplers(&[]),
            vk::DescriptorSetLayoutBinding::default()
                .binding(3)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT),
            //.immutable_samplers(&[]),
        ];

        let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

        unsafe {
            device
                .create_descriptor_set_layout(&create_info, None)
                .unwrap()
        }
    }

    fn create_deferred_lightning_set(
        &self,
        descriptor_pool: vk::DescriptorPool,
        deferred_lightning_descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> vk::DescriptorSet {
        let set_layouts = [deferred_lightning_descriptor_set_layout];
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&set_layouts);

        unsafe {
            self.device
                .allocate_descriptor_sets(&allocate_info)
                .unwrap()[0]
        }
    }

    // TODO: verify
    fn create_pipeline_layout(
        device: &Device,
        set_layouts: &[vk::DescriptorSetLayout],
        push_constant_ranges: &[vk::PushConstantRange],
    ) -> vk::PipelineLayout {
        let create_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(set_layouts)
            .push_constant_ranges(push_constant_ranges);

        unsafe { device.create_pipeline_layout(&create_info, None).unwrap() }
    }

    fn create_deferred_pipeline(
        device: &Device,
        shader_manager: &mut ShaderManager,
        render_area: vk::Rect2D,
        deferred_pipeline_layout: vk::PipelineLayout,
    ) -> vk::Pipeline {
        if let Some(shader) = shader_manager.get_shader("Deferred") {
            let vertex_binding_descriptions = [
                vk::VertexInputBindingDescription::default()
                    .binding(0)
                    .stride(std::mem::size_of::<nalgebra::Vector3<f32>>() as u32)
                    .input_rate(vk::VertexInputRate::VERTEX),
                vk::VertexInputBindingDescription::default()
                    .binding(1)
                    .stride(std::mem::size_of::<nalgebra::Vector3<f32>>() as u32)
                    .input_rate(vk::VertexInputRate::VERTEX),
            ];

            let vertex_input_attribute_descriptions = [
                vk::VertexInputAttributeDescription::default()
                    .location(0)
                    .binding(0)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(0),
                vk::VertexInputAttributeDescription::default()
                    .location(1)
                    .binding(1)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(0),
            ];

            let color_blend_attachments = [
                Self::create_pipeline_color_blend_attachment_state(),
                Self::create_pipeline_color_blend_attachment_state(),
                Self::create_pipeline_color_blend_attachment_state(),
            ];

            // TODO: manage format
            let color_attachemnt_formats = [
                deferred_renderpass_consts::COLOR,
                deferred_renderpass_consts::NORMAL,
                deferred_renderpass_consts::POSITION,
            ];

            return Self::create_pipeline(
                device,
                render_area,
                &shader,
                &vertex_binding_descriptions,
                &vertex_input_attribute_descriptions,
                &color_blend_attachments,
                &color_attachemnt_formats,
                deferred_pipeline_layout,
                vk::CullModeFlags::BACK,
            );
        }

        vk::Pipeline::null()
    }

    fn create_deferred_lightning_pipeline(
        device: &Device,
        shader_manager: &mut ShaderManager,
        render_area: vk::Rect2D,
        swapchain_format: vk::Format,
        deferred_lightning_pipeline_layout: vk::PipelineLayout,
    ) -> vk::Pipeline {
        if let Some(shader) = shader_manager.get_shader("DeferredLightning") {
            let color_blend_attachments = [Self::create_pipeline_color_blend_attachment_state()];

            let color_attachemnt_formats = [swapchain_format];

            return Self::create_pipeline(
                device,
                render_area,
                &shader,
                &[],
                &[],
                &color_blend_attachments,
                &color_attachemnt_formats,
                deferred_lightning_pipeline_layout,
                vk::CullModeFlags::BACK,
            );
        }

        vk::Pipeline::null()
    }

    fn create_shadow_map_pipeline(
        device: &Device,
        shader_manager: &mut ShaderManager,
        render_area: vk::Rect2D,
        shadow_map_layout: vk::PipelineLayout,
    ) -> vk::Pipeline {
        if let Some(shader) = shader_manager.get_shader("ShadowMap") {
            let vertex_binding_descriptions = [vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(std::mem::size_of::<nalgebra::Vector3<f32>>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX)];

            let vertex_input_attribute_descriptions =
                [vk::VertexInputAttributeDescription::default()
                    .location(0)
                    .binding(0)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(0)];

            let color_blend_attachments = [
            //create_pipeline_color_blend_attachment_state(),
        ];

            // TODO: manage format
            let color_attachemnt_formats = [
                // TODO: ShadowMapDefinitions
                //deferred_renderpass_consts::DEPTH,
            ];

            return Self::create_pipeline(
                device,
                render_area,
                &shader,
                &vertex_binding_descriptions,
                &vertex_input_attribute_descriptions,
                &color_blend_attachments,
                &color_attachemnt_formats,
                shadow_map_layout,
                vk::CullModeFlags::BACK,
            );
        }

        vk::Pipeline::null()
    }

    fn create_pipeline(
        device: &Device,
        render_area: vk::Rect2D,
        shader: &Shader,
        vertex_binding_descriptions: &[vk::VertexInputBindingDescription],
        vertex_input_attribute_descriptions: &[vk::VertexInputAttributeDescription],
        color_blend_attachments: &[vk::PipelineColorBlendAttachmentState],
        color_attachemnt_formats: &[vk::Format],
        pipeline_layout: vk::PipelineLayout,
        cull_mode: vk::CullModeFlags,
    ) -> vk::Pipeline {
        let shader_stage_create_infos = [
            Self::create_pipeline_shader_stage_create_info(
                vk::ShaderStageFlags::VERTEX,
                shader.vert,
            ),
            Self::create_pipeline_shader_stage_create_info(
                vk::ShaderStageFlags::FRAGMENT,
                shader.frag,
            ),
        ];

        let vertex_input_state = Self::create_pipeline_vertex_input_state_create_info(
            vertex_binding_descriptions,
            vertex_input_attribute_descriptions,
        );

        let input_assembly_state = Self::create_pipeline_input_assembly_state_create_info();
        let tessellation_state = Self::create_pipeline_tessellation_state_create_info();

        let viewports = [vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(render_area.extent.width as f32)
            .height(render_area.extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)];
        let render_areas = [render_area];

        let viewport_state = Self::create_viewport_state_create_info(&viewports, &render_areas);
        let rasterization_state = Self::create_rasterization_state_create_info(&cull_mode);
        let multisample_state = Self::create_pipeline_multisample_state_create_info();
        let depth_stencil_state = Self::create_pipeline_depth_stencil_state_create_info();
        let color_blend_state =
            Self::create_pipeline_color_blend_state_create_info(color_blend_attachments);
        let dynamic_state = Self::create_pipeline_dynamic_state_create_info();
        let mut pipeline_rendering_create_info_khr =
            Self::create_pipeline_rendering_create_info_khr(color_attachemnt_formats);

        let create_info = vk::GraphicsPipelineCreateInfo::default()
            .push_next(&mut pipeline_rendering_create_info_khr)
            .stages(&shader_stage_create_infos)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .tessellation_state(&tessellation_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(vk::RenderPass::null())
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(0);

        unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
                .unwrap()[0]
        }
    }

    fn create_pipeline_shader_stage_create_info(
        shader_stage: vk::ShaderStageFlags,
        module: vk::ShaderModule,
    ) -> vk::PipelineShaderStageCreateInfo<'static> {
        vk::PipelineShaderStageCreateInfo::default()
            .stage(shader_stage)
            .module(module)
            .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") })
    }

    fn create_pipeline_vertex_input_state_create_info<'a>(
        vertex_binding_descriptions: &'a [vk::VertexInputBindingDescription],
        vertex_attribute_descriptions: &'a [vk::VertexInputAttributeDescription],
    ) -> vk::PipelineVertexInputStateCreateInfo<'a> {
        vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(vertex_binding_descriptions)
            .vertex_attribute_descriptions(vertex_attribute_descriptions)
    }

    fn create_rasterization_state_create_info(
        cull_mode: &vk::CullModeFlags,
    ) -> vk::PipelineRasterizationStateCreateInfo {
        vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(cull_mode.clone())
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0)
            .line_width(1.0)
    }

    fn create_pipeline_input_assembly_state_create_info(
    ) -> vk::PipelineInputAssemblyStateCreateInfo<'static> {
        vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
    }

    fn create_pipeline_tessellation_state_create_info(
    ) -> vk::PipelineTessellationStateCreateInfo<'static> {
        vk::PipelineTessellationStateCreateInfo::default().patch_control_points(0)
    }

    fn create_viewport_state_create_info<'a>(
        viewports: &'a [vk::Viewport],
        render_areas: &'a [vk::Rect2D],
    ) -> vk::PipelineViewportStateCreateInfo<'a> {
        // TODO: scissors?
        vk::PipelineViewportStateCreateInfo::default()
            .viewports(viewports)
            .scissors(render_areas)
    }

    fn create_pipeline_multisample_state_create_info(
    ) -> vk::PipelineMultisampleStateCreateInfo<'static> {
        vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(0.0)
            .sample_mask(&[])
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false)
    }

    fn create_pipeline_depth_stencil_state_create_info(
    ) -> vk::PipelineDepthStencilStateCreateInfo<'static> {
        vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(true)
            .stencil_test_enable(true)
            .front(vk::StencilOpState::default())
            .back(vk::StencilOpState::default())
            .min_depth_bounds(0.0)
            .max_depth_bounds(0.0)
    }

    fn create_pipeline_color_blend_attachment_state() -> vk::PipelineColorBlendAttachmentState {
        vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::default())
            .dst_color_blend_factor(vk::BlendFactor::default())
            .color_blend_op(vk::BlendOp::default())
            .src_alpha_blend_factor(vk::BlendFactor::default())
            .dst_alpha_blend_factor(vk::BlendFactor::default())
            .alpha_blend_op(vk::BlendOp::default())
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
    }

    fn create_pipeline_color_blend_state_create_info(
        color_blend_attachments: &[vk::PipelineColorBlendAttachmentState],
    ) -> vk::PipelineColorBlendStateCreateInfo {
        vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::default())
            .attachments(color_blend_attachments)
    }

    fn create_pipeline_dynamic_state_create_info() -> vk::PipelineDynamicStateCreateInfo<'static> {
        vk::PipelineDynamicStateCreateInfo::default()
    }

    fn create_pipeline_rendering_create_info_khr(
        color_attachment_formats: &[vk::Format],
    ) -> vk::PipelineRenderingCreateInfoKHR {
        vk::PipelineRenderingCreateInfoKHR::default()
            .color_attachment_formats(color_attachment_formats)
            .depth_attachment_format(deferred_renderpass_consts::DEPTH)
            .stencil_attachment_format(vk::Format::UNDEFINED)
    }
}
