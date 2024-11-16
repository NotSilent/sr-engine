use std::{collections::HashMap, fs::File};

use ash::{util, vk, Device};

// TODO: Compile ti SPIR-V during runtime

#[derive(Clone)]
pub struct Shader {
    pub vert: vk::ShaderModule,
    pub frag: vk::ShaderModule,
}

pub struct ShaderManager {
    device: Device,
    compiler: shaderc::Compiler,
    shaders: HashMap<String, Shader>,
}

impl ShaderManager {
    pub fn new(device: Device) -> Self {
        let compiler = shaderc::Compiler::new().unwrap();
        let shaders = HashMap::new();

        Self {
            device,
            compiler,
            shaders,
        }
    }

    pub fn destroy(&mut self) {
        for (_, shader) in &self.shaders {
            unsafe {
                self.device.destroy_shader_module(shader.vert, None);
                self.device.destroy_shader_module(shader.frag, None);
            }
        }
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

    // TODO: return Option<&Shader>?
    pub fn get_shader(&mut self, name: &str) -> Option<Shader> {
        if name.is_empty() {
            return None;
        }

        if let Some(shader) = self.shaders.get(name) {
            return Some(shader.clone());
        }

        let vert = Self::create_shader_module(&self.device, name, vk::ShaderStageFlags::VERTEX);
        let frag = Self::create_shader_module(&self.device, name, vk::ShaderStageFlags::FRAGMENT);

        Some(Shader { vert, frag })
    }
}
