use nalgebra::Matrix4;
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
