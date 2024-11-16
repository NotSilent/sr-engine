use nalgebra::{Matrix4, Vector3};
use std::slice;

#[repr(C)]
pub struct PushConstantsData {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    projection: Matrix4<f32>,
}

impl PushConstantsData {
    pub fn new(model: &Matrix4<f32>, view: &Matrix4<f32>, projection: &Matrix4<f32>) -> Self {
        Self {
            model: *model,
            view: *view,
            projection: *projection,
        }
    }

    pub fn get(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const PushConstantsData as *const u8,
                std::mem::size_of::<PushConstantsData>(),
            )
        }
    }
}

#[repr(C)]
pub struct LightningPushConstantsData {
    light_space: Matrix4<f32>,
    view: Vector3<f32>,
}

impl LightningPushConstantsData {
    pub fn new(light_space: &Matrix4<f32>, view: &Vector3<f32>) -> Self {
        Self {
            light_space: *light_space,
            view: *view,
        }
    }

    pub fn get(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self as *const LightningPushConstantsData as *const u8,
                std::mem::size_of::<LightningPushConstantsData>(),
            )
        }
    }
}
