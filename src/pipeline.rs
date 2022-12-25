use ash::util;
use ash::vk::Rect2D;
use ash::{vk, Device};
use std::ffi::CStr;
use std::fs::File;

pub struct Pipeline {
    handle: vk::Pipeline,
}

impl Pipeline {
    #[inline(always)]
    pub fn get(&self) -> vk::Pipeline {
        self.handle
    }

    fn load_shader_code(name: &str, shader_stage: vk::ShaderStageFlags) -> Vec<u32> {
        let extension = match shader_stage {
            vk::ShaderStageFlags::VERTEX => "vert",
            vk::ShaderStageFlags::FRAGMENT => "frag",
            _ => "invalid",
        };
        let _file_name = format!("{}.{}.spv", name, extension);
        let file_path = format!("shaders/{}.{}.spv", name, extension);
        let mut file = File::open(file_path).unwrap();

        util::read_spv(&mut file).unwrap()
    }

    fn create_shader_module(
        device: &Device,
        name: &str,
        shader_stage: vk::ShaderStageFlags,
    ) -> vk::ShaderModule {
        let code = Self::load_shader_code(name, shader_stage);
        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(&code[..])
            .build();

        unsafe { device.create_shader_module(&create_info, None).unwrap() }
    }

    pub fn new(
        device: &Device,
        pipeline_layout: vk::PipelineLayout,
        render_area: Rect2D,
        color_attachment_formats: &[vk::Format],
    ) -> Pipeline {
        let mut pipeline_rendering_create_info = vk::PipelineRenderingCreateInfo::builder()
            .color_attachment_formats(color_attachment_formats)
            .build();

        let vertex_module =
            Self::create_shader_module(&device, "base", vk::ShaderStageFlags::VERTEX);
        let fragment_module =
            Self::create_shader_module(&device, "base", vk::ShaderStageFlags::FRAGMENT);

        let pipeline_stages = [
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_module)
                .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") })
                .build(),
            vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module)
                .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") })
                .build(),
        ];

        let vertex_binding_descriptions: [vk::VertexInputBindingDescription; 0] = [];
        // [vk::VertexInputBindingDescription::builder()
        // .binding(0)
        // .input_rate(vk::VertexInputRate::VERTEX)
        // .stride(mem::size_of::<f32>() as u32)
        // .build()];

        let vertex_attribute_descriptions: [vk::VertexInputAttributeDescription; 0] = [];
        // = [vk::VertexInputAttributeDescription::builder()
        // .format(vk::Format::R32G32_SFLOAT)
        // .binding(0)
        // .location(0)
        // .offset(0)
        // .build()];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_binding_descriptions)
            .build();

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .build();

        let tesselation_state = vk::PipelineTessellationStateCreateInfo::default();

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .scissors(&[render_area])
            .viewports(&[vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(render_area.extent.width as f32)
                .height(render_area.extent.height as f32)
                .min_depth(0.0)
                .max_depth(1.0)
                .build()])
            .build();

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0)
            .line_width(1.0)
            .build();

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(0.0)
            //.sample_mask()
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false)
            .build();

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .front(vk::StencilOpState::default())
            .back(vk::StencilOpState::default())
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .build();

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::SRC_COLOR)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_DST_COLOR)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ZERO)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .build()];

        let color_blending_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default();

        let create_info = vk::GraphicsPipelineCreateInfo::builder()
            .push_next(&mut pipeline_rendering_create_info)
            .flags(vk::PipelineCreateFlags::empty())
            .stages(&pipeline_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .tessellation_state(&tesselation_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blending_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .build();

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
                .unwrap()
        };

        Pipeline {
            handle: pipeline[0],
        }
    }
}
