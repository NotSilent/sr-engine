use approx::relative_eq;
use nalgebra::Matrix4;
use nalgebra::Vector3;

struct Face {
    top_left: nalgebra::Vector3<f32>,
    top_right: nalgebra::Vector3<f32>,
    bottom_left: nalgebra::Vector3<f32>,
    bottom_right: nalgebra::Vector3<f32>,
}

pub struct PatchedSphere {
    pub indices: Vec<u32>,
    pub positions: Vec<nalgebra::Vector3<f32>>,
    pub normals: Vec<nalgebra::Vector3<f32>>,
}

impl PatchedSphere {
    pub fn new(subdivisions: u32) -> Self {
        let forward_face = Face {
            top_left: nalgebra::Vector3::new(-1.0, 1.0, 1.0),
            top_right: nalgebra::Vector3::new(1.0, 1.0, 1.0),
            bottom_left: nalgebra::Vector3::new(-1.0, -1.0, 1.0),
            bottom_right: nalgebra::Vector3::new(1.0, -1.0, 1.0),
        };

        let back_face = Face {
            top_left: nalgebra::Vector3::new(1.0, 1.0, -1.0),
            top_right: nalgebra::Vector3::new(-1.0, 1.0, -1.0),
            bottom_left: nalgebra::Vector3::new(1.0, -1.0, -1.0),
            bottom_right: nalgebra::Vector3::new(-1.0, -1.0, -1.0),
        };

        let left_face = Face {
            top_left: nalgebra::Vector3::new(-1.0, 1.0, -1.0),
            top_right: nalgebra::Vector3::new(-1.0, 1.0, 1.0),
            bottom_left: nalgebra::Vector3::new(-1.0, -1.0, -1.0),
            bottom_right: nalgebra::Vector3::new(-1.0, -1.0, 1.0),
        };

        let right_face = Face {
            top_left: nalgebra::Vector3::new(1.0, 1.0, 1.0),
            top_right: nalgebra::Vector3::new(1.0, 1.0, -1.0),
            bottom_left: nalgebra::Vector3::new(1.0, -1.0, 1.0),
            bottom_right: nalgebra::Vector3::new(1.0, -1.0, -1.0),
        };

        let top_face = Face {
            top_left: nalgebra::Vector3::new(-1.0, 1.0, -1.0),
            top_right: nalgebra::Vector3::new(1.0, 1.0, -1.0),
            bottom_left: nalgebra::Vector3::new(-1.0, 1.0, 1.0),
            bottom_right: nalgebra::Vector3::new(1.0, 1.0, 1.0),
        };

        let bottom_face = Face {
            top_left: nalgebra::Vector3::new(-1.0, -1.0, 1.0),
            top_right: nalgebra::Vector3::new(1.0, -1.0, 1.0),
            bottom_left: nalgebra::Vector3::new(-1.0, -1.0, -1.0),
            bottom_right: nalgebra::Vector3::new(1.0, -1.0, -1.0),
        };

        let mut faces = vec![
            forward_face,
            back_face,
            left_face,
            right_face,
            top_face,
            bottom_face,
        ];

        for _ in 0..subdivisions {
            faces = Self::generate_subdivided_faces(faces);
        }

        let mut indices = vec![];
        let mut positions = vec![];

        for face in &faces {
            let top_left = face.top_left.normalize();
            let top_right = face.top_right.normalize();
            let bottom_left = face.bottom_left.normalize();
            let bottom_right = face.bottom_right.normalize();

            let top_left_index = Self::get_index_of_vertex(&top_left, &mut positions);
            let top_right_index = Self::get_index_of_vertex(&top_right, &mut positions);
            let bottom_left_index = Self::get_index_of_vertex(&bottom_left, &mut positions);
            let bottom_right_index = Self::get_index_of_vertex(&bottom_right, &mut positions);

            // Second triangle
            indices.push(top_left_index);
            indices.push(top_right_index);
            indices.push(bottom_left_index);

            // First triangle
            indices.push(bottom_left_index);
            indices.push(top_right_index);
            indices.push(bottom_right_index);
        }

        let normals = positions.clone();

        Self {
            indices,
            positions,
            normals,
        }
    }

    fn generate_subdivided_faces(faces: Vec<Face>) -> Vec<Face> {
        let mut new_faces = vec![];
        for face in &faces {
            let top_midpoint = (face.top_left + face.top_right) / 2.0;
            let bottom_midpoint = (face.bottom_left + face.bottom_right) / 2.0;
            let left_midpoint = (face.top_left + face.bottom_left) / 2.0;
            let right_midpoint = (face.top_right + face.bottom_right) / 2.0;
            let center = (face.top_left + face.bottom_right) / 2.0;

            let top_left = Face {
                top_left: face.top_left,
                top_right: top_midpoint,
                bottom_left: left_midpoint,
                bottom_right: center,
            };

            let top_right = Face {
                top_left: top_midpoint,
                top_right: face.top_right,
                bottom_left: center,
                bottom_right: right_midpoint,
            };

            let bottom_left = Face {
                top_left: left_midpoint,
                top_right: center,
                bottom_left: face.bottom_left,
                bottom_right: bottom_midpoint,
            };

            let bottom_right = Face {
                top_left: center,
                top_right: right_midpoint,
                bottom_left: bottom_midpoint,
                bottom_right: face.bottom_right,
            };

            new_faces.push(top_left);
            new_faces.push(top_right);
            new_faces.push(bottom_left);
            new_faces.push(bottom_right);
        }

        new_faces
    }

    fn get_index_of_vertex(
        vertex: &nalgebra::Vector3<f32>,
        positions: &mut Vec<nalgebra::Vector3<f32>>,
    ) -> u32 {
        if let Some(position) = positions
            .iter()
            .position(|elem| relative_eq![elem, vertex, epsilon = 0.0001])
        {
            return position as u32;
        }

        let new_index = positions.len() as u32;
        positions.push(*vertex);

        new_index
    }
}
