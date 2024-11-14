use crate::vertex::Vertex;
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
        let create_info = vk::ShaderModuleCreateInfo::default().code(&code[..]);

        unsafe { device.create_shader_module(&create_info, None).unwrap() }
    }

    pub fn new(
        device: &Device,
        pipeline_layout: vk::PipelineLayout,
        render_area: Rect2D,
        color_attachment_formats: &[vk::Format],
    ) -> Pipeline {
        let mut pipeline_rendering_create_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(color_attachment_formats);

        let vertex_module =
            Self::create_shader_module(&device, "base", vk::ShaderStageFlags::VERTEX);
        let fragment_module =
            Self::create_shader_module(&device, "base", vk::ShaderStageFlags::FRAGMENT);

        let pipeline_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_module)
                .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_module)
                .name(unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }),
        ];

        let vertex_binding_descriptions = [vk::VertexInputBindingDescription::default()
            .binding(0)
            .input_rate(vk::VertexInputRate::VERTEX)
            .stride(std::mem::size_of::<Vertex>() as u32)];

        let vertex_attribute_descriptions = [vk::VertexInputAttributeDescription::default()
            .format(vk::Format::R32G32B32_SFLOAT)
            .binding(0)
            .location(0)
            .offset(0)];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_attribute_descriptions(&vertex_attribute_descriptions)
            .vertex_binding_descriptions(&vertex_binding_descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let tesselation_state = vk::PipelineTessellationStateCreateInfo::default();

        let scissors = [render_area];

        let viewports = [vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(render_area.extent.width as f32)
            .height(render_area.extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0)];

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .scissors(&scissors)
            .viewports(&viewports);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false)
            .depth_bias_constant_factor(0.0)
            .depth_bias_clamp(0.0)
            .depth_bias_slope_factor(0.0)
            .line_width(1.0);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
            .sample_shading_enable(false)
            .min_sample_shading(0.0)
            //.sample_mask()
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .front(vk::StencilOpState::default())
            .back(vk::StencilOpState::default())
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0);

        let color_blend_attachments = [vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(false)
            .src_color_blend_factor(vk::BlendFactor::SRC_COLOR)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_DST_COLOR)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ZERO)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA)];

        let color_blending_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::CLEAR)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default();

        let create_info = vk::GraphicsPipelineCreateInfo::default()
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
            .layout(pipeline_layout);

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[create_info], None)
                .unwrap()
        };

        unsafe {
            device.destroy_shader_module(vertex_module, None);
            device.destroy_shader_module(fragment_module, None);
        }

        Pipeline {
            handle: pipeline[0],
        }
    }
}
