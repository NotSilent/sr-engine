use nalgebra::{Matrix4, Vector3};

// 1/((w/h)tan(theta/2))) 0            0       0
// 0                      tan(theta/2) 0       0
// 0                      0            f/(f-n) -fn/(f-n)
// 0                      0            1       0

// Vulkan
// x right
// y bottom
// z forward
//
// Engine
// x right
// y forward
// z up

pub struct Camera {
    transform: Matrix4<f32>,
    projection: Matrix4<f32>,
}

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
        let mut transform = Matrix4::new_scaling(1.0);
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
        Matrix4::<f32>::new(
            1.0 / ((width / height) * f32::tan(fov / 2.0)),
            0.0,
            0.0,
            0.0,
            0.0,
            f32::tan(fov / 2.0),
            0.0,
            0.0,
            0.0,
            0.0,
            far / (far - near),
            -far * near / (far - near),
            0.0,
            0.0,
            1.0,
            0.0,
        )
    }

    pub fn get_projection(&self) -> &Matrix4<f32> {
        &self.projection
    }

    pub fn get_view(&self) -> Matrix4<f32> {
        static TO_VULKAN_COORDINATE_SYSTEM: Matrix4<f32> = Matrix4::new(
            1.0, 0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        );

        TO_VULKAN_COORDINATE_SYSTEM * self.transform.try_inverse().unwrap()
    }
}
