use nalgebra::{Matrix4, Vector3};

// Vulkan
// x right
// y bottom
// z forward
//
// Engine
// x right
// y up
// z forward
//
// nalgebra
// x right
// y up
// z forward

pub struct Camera {
    transform: Matrix4<f32>,
    projection: Matrix4<f32>,
}

const TO_VULKAN_COORDINATE_SYSTEM: Matrix4<f32> = Matrix4::new(
    -1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
);

impl Camera {
    pub fn new(
        x: f32,
        y: f32,
        z: f32,
        width: f32,
        height: f32,
        fov: f32,
        near: f32,
        far: f32,
    ) -> Self {
        let mut transform = Matrix4::identity();
        transform.append_translation_mut(&Vector3::new(x, y, z));

        Self {
            transform,
            projection: Self::calculate_projection(width, height, fov, near, far),
        }
    }

    fn calculate_projection(
        width: f32,
        height: f32,
        fov: f32,
        near: f32,
        far: f32,
    ) -> Matrix4<f32> {
        let perspective = nalgebra::Matrix4::new_perspective(width / height, fov, near, far);

        //TO_VULKAN_COORDINATE_SYSTEM * perspective
        perspective
    }

    pub fn get_projection(&self) -> &Matrix4<f32> {
        &self.projection
    }

    pub fn get_view(&self) -> Matrix4<f32> {
        //self.transform.try_inverse().unwrap()

        TO_VULKAN_COORDINATE_SYSTEM
            * nalgebra::Matrix4::look_at_rh(
                &nalgebra::Point3::new(0.0, 0.0, -5.0),
                &nalgebra::Point3::new(0.0, 0.0, 0.0),
                &Vector3::new(0.0, 1.0, 0.0),
            )
    }
}
