use glam::Vec3;

use crate::{IndexMesh, index_mesh::IndexTriangle};

pub struct Box {
    pub size: f32
}

impl Box {
    pub fn to_mesh(&self) -> IndexMesh {
        let mut mesh = IndexMesh::new();
        let half_size = self.size / 2.0;
        mesh.vertices = vec![
            Vec3::new(-half_size, -half_size, -half_size),
            Vec3::new(-half_size, -half_size, half_size),
            Vec3::new(-half_size, half_size, -half_size),
            Vec3::new(-half_size, half_size, half_size),
            Vec3::new(half_size, -half_size, -half_size),
            Vec3::new(half_size, -half_size, half_size),
            Vec3::new(half_size, half_size, -half_size),
            Vec3::new(half_size, half_size, half_size),
        ];
        mesh.triangles = vec![
            IndexTriangle(0, 1, 2),
            IndexTriangle(1, 3, 2),
            IndexTriangle(4, 6, 5),
            IndexTriangle(5, 6, 7),
            IndexTriangle(0, 2, 4),
            IndexTriangle(4, 2, 6),
            IndexTriangle(1, 5, 3),
            IndexTriangle(5, 7, 3),
            IndexTriangle(0, 4, 1),
            IndexTriangle(4, 5, 1),
            IndexTriangle(2, 3, 6),
            IndexTriangle(6, 3, 7),
        ];
        mesh
    }
}

// #[derive(Debug, PartialEq)]
// struct Cylinder {
//     height: f64,
//     radius: f64,
// }

// impl Cylinder {
//     fn to_mesh(&self, radius_seg: usize, ) -> IndexMesh {
//         todo!()
//     }
// }
