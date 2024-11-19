use ash::vk;
use nalgebra::Matrix4;

use crate::camera::Camera;

pub struct DirectionalLight {
    position: nalgebra::Vector3<f32>,
    target: nalgebra::Vector3<f32>,
    projection: nalgebra::Matrix4<f32>,
}

const ENGINE_TO_VULKAN_COORDINATE_SPACE: nalgebra::Matrix4<f32> = Matrix4::new(
    -1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
);

impl DirectionalLight {
    fn new(position: nalgebra::Vector3<f32>, target: nalgebra::Vector3<f32>) -> Self {
        Self {
            position,
            target,
            projection: nalgebra::Matrix4::new_orthographic(-20.0, 20.0, -20.0, 20.0, 0.001, 20.0),
        }
    }

    pub fn get_view(&self) -> nalgebra::Matrix4<f32> {
        // TODO: rh?
        ENGINE_TO_VULKAN_COORDINATE_SPACE
            * nalgebra::Matrix4::look_at_rh(
                &nalgebra::Point::from(self.position),
                &nalgebra::Point::from(self.target),
                &nalgebra::Vector3::new(0.0, 1.0, 0.0),
            )
    }

    pub fn get_projection(&self) -> nalgebra::Matrix4<f32> {
        self.projection
    }
}

#[derive(Clone)]
pub struct MeshData {
    pub index_count: u32,
    pub index_buffer: vk::Buffer,
    pub positions_buffer: vk::Buffer,
    pub attributes_buffer: vk::Buffer,
}

impl MeshData {
    pub fn new(
        index_count: u32,
        index_buffer: vk::Buffer,
        positions_buffer: vk::Buffer,
        attributes_buffer: vk::Buffer,
    ) -> Self {
        Self {
            index_count,
            index_buffer,
            positions_buffer,
            attributes_buffer,
        }
    }
}

pub struct DrawCall {
    pub mesh: MeshData,
    pub model: Matrix4<f32>,
    pub pipeline: vk::Pipeline, // TODO: Why tf part of DrawCall?
}

impl DrawCall {
    pub fn new(mesh: &MeshData, model: Matrix4<f32>, pipeline: vk::Pipeline) -> Self {
        Self {
            model,
            mesh: mesh.clone(),
            pipeline,
        }
    }
}

pub struct DrawData {
    pub directional_light: DirectionalLight,
    pub draw_calls: Vec<DrawCall>,
    pub view: Matrix4<f32>,
    pub projection: Matrix4<f32>,
    pub deferred_pipeline_layout: vk::PipelineLayout, // TODO: Why tf part of DrawData?
}

impl DrawData {
    pub fn new(camera: &Camera, deferred_pipeline_layout: vk::PipelineLayout) -> Self {
        Self {
            directional_light: DirectionalLight::new(
                nalgebra::Vector3::new(0.0, 0.0, -5.0),
                nalgebra::Vector3::new(0.0, 0.0, 0.0),
            ),
            draw_calls: vec![],
            view: camera.get_view(),
            projection: *camera.get_projection(),
            deferred_pipeline_layout,
        }
    }

    pub fn add_draw_call(&mut self, draw_call: DrawCall) {
        self.draw_calls.push(draw_call);
    }
}
