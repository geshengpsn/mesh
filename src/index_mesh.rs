use std::{collections::HashMap, fmt::Debug, slice::Iter};

use anyhow::Ok;
use glam::Vec3;

use crate::{
    bvh::{BuildBvhOption, BVH},
    half_edge::HalfEdgeMesh,
};

#[derive(Clone, Copy)]
pub struct IndexTriangle(pub usize, pub usize, pub usize);

impl Debug for IndexTriangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}, {}, {})", self.0, self.1, self.2))?;
        std::fmt::Result::Ok(())
    }
}

#[derive(Debug)]
pub struct IndexMesh {
    pub(crate) vertices: Vec<Vec3>,
    pub(crate) triangles: Vec<IndexTriangle>,
}

impl From<&HalfEdgeMesh> for IndexMesh {
    fn from(value: &HalfEdgeMesh) -> Self {
        let mut mesh = IndexMesh::new();
        let mut vertex_map = HashMap::new();
        for (index, (uuid, v)) in value.vertices.iter().enumerate() {
            mesh.vertices.push(v.geometry);
            vertex_map.insert(uuid, index);
        }
        for (_, f) in value.faces.iter() {
            let (v1, v2, v3) = value.find_vertex_in_face(f);
            mesh.triangles.push(IndexTriangle(
                *vertex_map.get(&v1.uuid).unwrap(),
                *vertex_map.get(&v2.uuid).unwrap(),
                *vertex_map.get(&v3.uuid).unwrap(),
            ));
        }
        mesh
    }
}

impl IndexMesh {
    pub fn new() -> Self {
        IndexMesh {
            vertices: vec![],
            triangles: vec![],
        }
    }

    pub fn vertices(&self) -> Iter<Vec3> {
        self.vertices.iter()
    }

    pub fn triangles(&self) -> Iter<IndexTriangle> {
        self.triangles.iter()
    }

    pub fn from_stl<F: std::io::Read + std::io::Seek>(f: &mut F) -> anyhow::Result<Self> {
        let mut res = IndexMesh::new();
        let mesh = stl_io::read_stl(f)?;
        mesh.vertices.into_iter().for_each(|vertex| {
            res.vertices
                .push(Vec3::new(vertex[0], vertex[1], vertex[2]));
        });
        mesh.faces.into_iter().for_each(|face| {
            res.triangles.push(IndexTriangle(
                face.vertices[0],
                face.vertices[1],
                face.vertices[2],
            ));
        });
        Ok(res)
    }

    pub fn to_stl<F: std::io::Write>(&self, f: &mut F) -> anyhow::Result<()> {
        let mut triangles = Vec::new();
        for triangle in self.triangles.iter() {
            let normal = (self.vertices[triangle.1] - self.vertices[triangle.0])
                .cross(self.vertices[triangle.2] - self.vertices[triangle.0])
                .normalize();
            triangles.push(stl_io::Triangle {
                normal: stl_io::Vertex::new(normal.into()),
                vertices: [
                    stl_io::Vertex::new(self.vertices[triangle.0].into()),
                    stl_io::Vertex::new(self.vertices[triangle.1].into()),
                    stl_io::Vertex::new(self.vertices[triangle.2].into()),
                ],
            });
        }
        stl_io::write_stl(f, triangles.iter())?;
        Ok(())
    }

    pub fn build_bvh(&self, option: BuildBvhOption) -> BVH<3, &IndexTriangle> {
        let a = self
            .triangles()
            .map(|t| {
                (
                    t,
                    [self.vertices[t.0], self.vertices[t.1], self.vertices[t.2]],
                )
            })
            .collect::<Vec<_>>();
        let a = BVH::build(option, a);
        a.transfrom::<&IndexTriangle>()
    }
}

impl<'a> From<(&'a IndexTriangle, [Vec3; 3])> for &'a IndexTriangle {
    fn from(value: (&'a IndexTriangle, [Vec3; 3])) -> Self {
        value.0
    }
}

#[cfg(test)]
mod index_mesh_tests {
    use std::fs::File;

    use glam::Vec3;

    use super::IndexMesh;

    #[test]
    fn test_from_stl() {
        let mut reader = ::std::io::Cursor::new(
            b"solid foobar
                   facet normal 0.1 0.2 0.3
                       outer loop
                           vertex 1 2 3
                           vertex 4 5 6e-15
                           vertex 7 8 9.87654321
                       endloop
                   endfacet
                   endsolid foobar"
                .to_vec(),
        );
        let res = IndexMesh::from_stl(&mut reader);
        assert!(res.is_ok());
        let mesh = res.unwrap();
        assert_eq!(mesh.vertices.len(), 3);
        assert_eq!(mesh.triangles.len(), 1);
        assert_eq!(mesh.vertices[mesh.triangles[0].0], Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(
            mesh.vertices[mesh.triangles[0].1],
            Vec3::new(4.0, 5.0, 6e-15)
        );
        assert_eq!(
            mesh.vertices[mesh.triangles[0].2],
            Vec3::new(7.0, 8.0, 9.87654321)
        );
    }

    #[test]
    fn test_to_stl() {
        let mut writer = ::std::io::Cursor::new(Vec::new());
        let mut mesh = IndexMesh::new();
        mesh.vertices.push(Vec3::new(1.0, 2.0, 3.0));
        mesh.vertices.push(Vec3::new(4.0, 5.0, 6e-15));
        mesh.vertices.push(Vec3::new(7.0, 8.0, 9.87654321));
        mesh.triangles.push(super::IndexTriangle(0, 1, 2));
        let res = mesh.to_stl(&mut writer);
        assert!(res.is_ok());
    }

    #[test]
    fn test_build_bvh() {
        let mesh = IndexMesh::from_stl(&mut File::open("assets/bunny.stl").unwrap()).unwrap();
        let start = std::time::Instant::now();
        let _bvh = mesh.build_bvh(Default::default());
        let end = std::time::Instant::now();
        println!("build bvh: {:?}", end - start);
        // let res = bvh.intersect(g, &mesh);
        // println!("{:?}", bvh);
    }
}
